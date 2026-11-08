use std::time::{Duration, Instant};

use toolbox_plan::procedure::{Command, Program};
use toolbox_plan::protocol::{Event, Failure, Outcome};
use toolbox_plan::{Password, SecretInput, StepId, UserName};

use super::{FEED_DEADLINE, FeedError, report, run};
use crate::pty_runner::Environment;

const SECRET: &str = "correct horse battery staple";
const SHELL: &str = "/bin/sh";

fn input() -> SecretInput {
    Password::new(SECRET.to_owned())
        .unwrap()
        .chpasswd_input(&UserName::new("nico").unwrap())
}

fn chpasswd() -> Command {
    Command::new(Program::Chpasswd)
}

fn fed(script: &str, args: &[&str]) -> std::process::Output {
    let command = chpasswd().args(["-c", script]).args(args.iter().copied());

    run(SHELL, &command, Environment::Clean, &input(), FEED_DEADLINE).unwrap()
}

#[test]
fn the_program_reads_the_secret_from_its_standard_input() {
    let output = fed(
        "read -r line; test \"$line\" = \"$1\"",
        &["sh", &format!("nico:{SECRET}")],
    );

    assert!(output.status.success());
}

#[test]
fn the_secret_is_in_no_argument_and_no_variable() {
    let output = fed("echo \"$@\"; set", &["sh", "seen"]);
    let printed = String::from_utf8_lossy(&output.stdout);

    assert!(printed.contains("seen"));
    assert!(!printed.contains(SECRET), "{printed}");
    assert!(printed.contains("LANG='C.UTF-8'") || printed.contains("LANG=C.UTF-8"));
    assert!(!printed.contains("HOME="), "the environment is not clean");
}

#[test]
fn a_program_that_echoes_its_input_is_scrubbed() {
    let mut events: Vec<Event> = Vec::new();
    let step = StepId::new(4);
    let output = fed(
        "read -r line; echo \"$line\"; echo \"bad line: $line\" >&2; exit 3",
        &[],
    );

    assert!(String::from_utf8_lossy(&output.stdout).contains(SECRET));

    let outcome = report(step, &chpasswd(), Ok(output), &input(), &mut events);

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::ExitCode { code: 3_i32 }
        }
    );
    assert_eq!(
        events,
        [Event::Output {
            step,
            data: "nico:<redacted>\r\nbad line: nico:<redacted>\r\n".to_owned()
        }]
    );
}

#[test]
fn a_quiet_success_reports_nothing() {
    let mut events: Vec<Event> = Vec::new();
    let output = fed("read -r line", &[]);

    assert_eq!(
        report(
            StepId::new(1),
            &chpasswd(),
            Ok(output),
            &input(),
            &mut events
        ),
        Outcome::Succeeded
    );
    assert_eq!(events, Vec::new());
}

#[test]
fn a_missing_program_fails_the_step_without_the_secret() {
    let mut events: Vec<Event> = Vec::new();
    let result = run(
        "/nonexistent/program",
        &chpasswd(),
        Environment::Clean,
        &input(),
        FEED_DEADLINE,
    );

    assert!(result.is_err());
    assert!(matches!(
        report(StepId::new(1), &chpasswd(), result, &input(), &mut events),
        Outcome::Failed {
            cause: Failure::Unknown
        }
    ));
    assert!(!format!("{events:?}").contains(SECRET));
}

/// A second passphrase prompt or a device that never answers would
/// otherwise hold the helper without an end.
#[test]
fn a_program_that_never_ends_is_killed_at_the_deadline() {
    let mut events: Vec<Event> = Vec::new();
    let step = StepId::new(6);
    let started = Instant::now();

    // Loops, since the clean `PATH` has no `sleep` everywhere. The one in
    // the background keeps the pipes open after the shell is gone.
    let script = "read -r line; echo \"$line\"; (while :; do :; done) & while :; do :; done";
    let command = chpasswd().args(["-c", script]);
    let result = run(
        SHELL,
        &command,
        Environment::Clean,
        &input(),
        Duration::from_millis(300),
    );

    assert!(started.elapsed() < Duration::from_secs(20));
    assert!(matches!(result, Err(FeedError::Late(_))), "{result:?}");
    assert_eq!(
        report(step, &chpasswd(), result, &input(), &mut events),
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    assert_eq!(
        events,
        [Event::Output {
            step,
            data: "chpasswd: it did not end within 0 seconds and was stopped\r\n".to_owned()
        }]
    );
}

#[test]
fn the_step_itself_runs_the_program_and_scrubs_its_answer() {
    let mut events: Vec<Event> = Vec::new();
    let outcome = super::feed(
        StepId::new(1),
        &chpasswd(),
        Environment::Clean,
        &input(),
        &mut events,
    );

    assert!(
        matches!(outcome, Outcome::Failed { cause: _ }),
        "chpasswd cannot set passwords as this user: {outcome:?}"
    );
    assert!(!format!("{events:?}").contains(SECRET));
}

/// The shell is gone but its background loop keeps the pipes open: the
/// step still ends at the deadline instead of waiting for the last byte.
#[test]
fn a_program_whose_children_outlive_it_is_stopped_at_the_deadline() {
    let command = chpasswd().args(["-c", "echo started; (while :; do :; done) &"]);
    let result = run(
        SHELL,
        &command,
        Environment::Clean,
        &input(),
        Duration::from_millis(300),
    );

    assert!(matches!(result, Err(FeedError::Late(_))), "{result:?}");
}
