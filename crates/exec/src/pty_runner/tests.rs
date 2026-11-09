mod late;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use toolbox_plan::procedure::{Builtin, Command, Program};
use toolbox_plan::protocol::{Answer, Event, Outcome, TerminalSize};
use toolbox_plan::{HomePath, Password, StepId, UserConfigEdit, UserName};
use toolbox_pty::{DEFAULT_SIZE, Exit, PtyCommand, Session};

use super::{Environment, Guard, PtyRunner, pty_command};
use crate::apply::Runner as _;
use crate::testing::printed;

/// Nanoseconds this thread spent on a CPU so far.
fn cpu_time() -> u64 {
    std::fs::read_to_string("/proc/thread-self/schedstat")
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn a_program_that_closed_its_output_is_waited_for_without_spinning() {
    let (_app, answers) = std::sync::mpsc::channel();
    let mut runner = PtyRunner::new(answers, Environment::Clean);
    let command =
        PtyCommand::new("/bin/sh").args(["-c", "exec </dev/null >/dev/null 2>&1; sleep 0.5"]);

    let mut session = Session::spawn(&command).unwrap();
    let before = cpu_time();

    runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();

    let spent = Duration::from_nanos(cpu_time() - before);

    assert!(spent < Duration::from_millis(100), "{spent:?} on the CPU");
}

/// What SIGTERM sets in the helper. The app is still there, so no answer
/// and no closed channel says to stop.
#[test]
fn a_set_stop_flag_cancels_the_running_program_and_the_steps_after_it() {
    let (_app, answers) = std::sync::mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let mut runner = PtyRunner::new(answers, Environment::Clean).stop_on(Arc::clone(&stop));
    let mut session =
        Session::spawn(&PtyCommand::new("/bin/sh").args(["-c", "read -r line"])).unwrap();

    let started = Instant::now();
    let signal = thread::Builder::new()
        .name("test-cancel".to_owned())
        .spawn(move || {
            thread::sleep(Duration::from_millis(200));
            stop.store(true, Ordering::Relaxed);
        })
        .unwrap();

    let exit = runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();

    signal.join().unwrap();

    assert_ne!(exit, Exit::Code(0));
    assert!(started.elapsed() < Duration::from_secs(20));
    assert!(runner.cancelled);
    assert_eq!(
        runner.run(
            StepId::new(2),
            &Command::new(Program::Pacman),
            &mut Vec::<Event>::new()
        ),
        Outcome::Cancelled
    );
}

#[test]
fn a_set_stop_flag_leaves_a_shielded_program_running_to_its_end() {
    let (_app, answers) = std::sync::mpsc::channel();
    let stop = Arc::new(AtomicBool::new(true));
    let mut runner = PtyRunner::new(answers, Environment::Clean).stop_on(stop);
    let mut session =
        Session::spawn(&PtyCommand::new("/bin/sh").args(["-c", "sleep 0.3; exit 0"])).unwrap();

    runner.guard = Guard::Shielded;

    let exit = runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();

    assert_eq!(exit, Exit::Code(0_i32));
    assert!(runner.cancelled);
}

fn runner_with(answers: Vec<Answer>) -> (PtyRunner, std::sync::mpsc::Sender<Answer>) {
    let (app, replies) = std::sync::mpsc::channel();

    for answer in answers {
        app.send(answer).unwrap();
    }

    (PtyRunner::new(replies, Environment::Clean), app)
}

/// An answer that arrives before the step runs cancels it unseen: nothing
/// starts, so there is nothing to stop.
#[test]
fn an_early_cancel_skips_the_step() {
    let (mut runner, _app) = runner_with(vec![Answer::Cancel]);

    assert_eq!(
        runner.run(
            StepId::new(1),
            &Command::new(Program::Pacman),
            &mut Vec::<Event>::new()
        ),
        Outcome::Cancelled
    );
    assert!(runner.cancelled);
}

/// The first Ctrl+C on the command line: the program runs to its end and
/// only the steps after it are skipped.
#[test]
fn a_finish_lets_the_running_program_end() {
    let (mut runner, _app) = runner_with(vec![Answer::Finish]);
    let mut session =
        Session::spawn(&PtyCommand::new("/bin/sh").args(["-c", "sleep 0.3"])).unwrap();

    let exit = runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();

    assert_eq!(exit, Exit::Code(0));
    assert!(!runner.cancelled);
    assert!(runner.finish_requested());
}

/// The second Ctrl+C: a cancel after a finish still stops the program now.
#[test]
fn a_cancel_after_a_finish_stops_the_running_program() {
    let (mut runner, _app) = runner_with(vec![Answer::Finish, Answer::Cancel]);
    let mut session =
        Session::spawn(&PtyCommand::new("/bin/sh").args(["-c", "read -r line"])).unwrap();

    let started = Instant::now();
    runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();

    assert!(runner.cancelled);
    assert!(started.elapsed() < Duration::from_secs(5));
}

/// Input with no running program is dropped, a resize is kept for the
/// next program.
#[test]
fn answers_without_a_program_fail_nothing() {
    let (mut runner, _app) = runner_with(vec![
        Answer::Input {
            data: "yes\n".to_owned(),
        },
        Answer::Resize {
            size: TerminalSize::from((100_u16, 30_u16)),
        },
    ]);

    // `pacman` runs or refuses to start depending on the system: either
    // way the step, not the answers, decides the outcome.
    let outcome = runner.run(
        StepId::new(1),
        &Command::new(Program::Pacman),
        &mut Vec::<Event>::new(),
    );

    assert!(matches!(
        outcome,
        Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }
    ));
    assert_eq!(runner.size.columns, 100_u16);
    assert_eq!(runner.size.rows, 30_u16);
}

/// A closed channel means the app is gone: the step stops like on a
/// cancel, and nothing keeps running as root without a UI.
#[test]
fn a_gone_app_cancels_the_step() {
    let (app, replies) = std::sync::mpsc::channel::<Answer>();
    drop(app);
    let mut runner = PtyRunner::new(replies, Environment::Clean);

    assert_eq!(
        runner.run(
            StepId::new(1),
            &Command::new(Program::Pacman),
            &mut Vec::<Event>::new()
        ),
        Outcome::Cancelled
    );
}

/// Typed input reaches the program in the terminal: the script echoes the
/// reply back into the output.
#[test]
fn typed_input_reaches_the_running_program() {
    let (mut runner, _app) = runner_with(vec![Answer::Input {
        data: "quit\n".to_owned(),
    }]);

    let mut session = Session::spawn(
        &PtyCommand::new("/bin/sh").args(["-c", "read -r line; echo \"got: $line\""]),
    )
    .unwrap();

    let mut events = Vec::<Event>::new();

    runner
        .pump(StepId::new(1), &mut session, &mut events)
        .unwrap();

    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::Output { step: _, data } if data.contains("got: quit")
        )),
        "{events:?}"
    );
}

#[test]
fn a_cancel_answer_stops_the_running_program() {
    let (app, replies) = std::sync::mpsc::channel();
    let mut runner = PtyRunner::new(replies, Environment::Clean);
    let mut session =
        Session::spawn(&PtyCommand::new("/bin/sh").args(["-c", "read -r line"])).unwrap();

    let started = Instant::now();
    let canceller = thread::Builder::new()
        .name("test-cancel".to_owned())
        .spawn(move || {
            thread::sleep(Duration::from_millis(200));
            app.send(Answer::Cancel).unwrap();
        })
        .unwrap();

    runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();
    canceller.join().unwrap();

    assert!(runner.cancelled);
    assert!(started.elapsed() < Duration::from_secs(20));
}

/// A program that never reads its terminal lets a long paste wait. The
/// cancel queued behind it must still stop the program at once.
#[test]
fn a_cancel_behind_a_paste_the_program_never_reads_still_stops_it() {
    let paste = Answer::Input {
        data: "x\n".repeat(32 * 1024),
    };

    let (mut runner, _app) = runner_with(vec![paste.clone(), paste, Answer::Cancel]);
    let mut session = Session::spawn(&PtyCommand::new("sleep").arg("20")).unwrap();
    let started = Instant::now();

    let exit = runner
        .pump(StepId::new(1), &mut session, &mut Vec::<Event>::new())
        .unwrap();

    assert_eq!(exit, Exit::Signal(2), "SIGINT ended it");
    assert!(runner.cancelled);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the paste held the cancel for {:?}",
        started.elapsed()
    );
}

#[test]
fn a_missing_program_fails_its_query() {
    let (mut runner, _app) = runner_with(Vec::new());

    // Without `pacman` the question fails. With it, usage text and exit
    // code 1 are no empty answer either.
    runner.query(&Command::new(Program::Pacman)).unwrap_err();
}

#[test]
fn run_always_resets_its_guard() {
    let (mut runner, _app) = runner_with(Vec::new());

    let outcome = runner.run_always(
        StepId::new(1),
        &Command::new(Program::Pacman),
        &mut Vec::<Event>::new(),
    );

    assert!(matches!(
        outcome,
        Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }
    ));
    assert_eq!(runner.guard, Guard::Cancellable);
}

#[test]
fn builtins_and_feeds_go_through_the_runner() {
    let (mut runner, _app) = runner_with(Vec::new());
    let builtin = Builtin::EditUserConfig {
        edit: UserConfigEdit::File {
            path: HomePath::new(".config/app.conf").unwrap(),
            content: "on = true\n".to_owned(),
        },
        on: true,
    };

    assert!(matches!(
        runner.builtin(StepId::new(1), &builtin, &mut Vec::<Event>::new()),
        Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }
    ));

    let (mut stopped, _stopped_app) = runner_with(vec![Answer::Cancel]);
    let input = Password::new("correct horse battery staple".to_owned())
        .unwrap()
        .chpasswd_input(&UserName::new("nico").unwrap());

    assert_eq!(
        stopped.feed(
            StepId::new(1),
            &Command::new(Program::Pacman),
            &input,
            &mut Vec::<Event>::new()
        ),
        Outcome::Cancelled
    );
}

#[test]
fn inherited_steps_keep_the_users_environment() {
    let home = std::env::var("HOME").expect("the tests run with a home");
    let echo = Command::new(Program::Pacman).args(["-c", "echo \"home=$HOME.\""]);
    let (mut runner, _app) = runner_with(Vec::new());
    let mut home_in = |environment| {
        let command = pty_command(Path::new("/bin/sh"), &echo, DEFAULT_SIZE, environment);
        let mut session = Session::spawn(&command).unwrap();
        let mut events = Vec::new();
        runner
            .pump(StepId::new(1), &mut session, &mut events)
            .unwrap();
        printed(&events)
    };

    assert!(home_in(Environment::Inherited).contains(&format!("home={home}.")));
    assert!(home_in(Environment::Clean).contains("home=."));
}
