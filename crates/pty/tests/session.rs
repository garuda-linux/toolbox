use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};
use std::{io, thread};

use toolbox_pty::{Chunk, Exit, PtyCommand, Session, TerminalSize, strip_ansi};

const PATIENCE: Duration = Duration::from_secs(10);

/// Set in the copy of this test binary that starts with Ctrl+C ignored.
const IGNORING: &str = "TOOLBOX_PTY_TEST_IGNORING";

fn shell(script: &str) -> PtyCommand {
    PtyCommand::new("sh").args(["-c", script])
}

/// Returns whether it closed.
fn read_until_closed(session: &Session, out: &mut String, deadline: Instant) -> bool {
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        match session.output().recv_timeout(left) {
            Ok(Chunk::Text(text)) => out.push_str(&text),
            Ok(Chunk::Closed) | Err(RecvTimeoutError::Disconnected) => return true,
            Err(RecvTimeoutError::Timeout) => return false,
        }
    }

    false
}

/// Without colour codes or carriage returns, and no trailing empty line for a waiting cursor.
fn plain_lines(out: &str) -> Vec<String> {
    let mut lines: Vec<String> = out
        .split('\n')
        .map(|line| strip_ansi(line.strip_suffix('\r').unwrap_or(line)))
        .collect();

    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }

    lines
}

fn read_while<W>(session: &Session, out: &mut String, deadline: Instant, waiting: W)
where
    W: Fn(&str) -> bool,
{
    while waiting(out) && Instant::now() < deadline {
        if let Ok(Chunk::Text(text)) = session.output().recv_timeout(Duration::from_millis(50)) {
            out.push_str(&text);
        }
    }
}

fn wait_exit(session: &mut Session, deadline: Instant) -> io::Result<Option<Exit>> {
    while Instant::now() < deadline {
        session.poll_cancel(Instant::now())?;
        if let Some(exit) = session.try_wait()? {
            return Ok(Some(exit));
        }

        thread::sleep(Duration::from_millis(20));
    }

    Ok(None)
}

fn finished(session: &mut Session) -> Option<Exit> {
    wait_exit(session, Instant::now() + PATIENCE).ok().flatten()
}

#[test]
fn coloured_output_reads_as_plain_text() {
    let mut session =
        Session::spawn(&PtyCommand::new("printf").arg("\\033[32mgreen\\033[0m and <plain>\\n"))
            .unwrap();

    let mut out = String::new();
    assert!(read_until_closed(
        &session,
        &mut out,
        Instant::now() + PATIENCE
    ));
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));
    assert_eq!(plain_lines(&out), ["green and <plain>"]);
}

#[test]
fn the_program_runs_in_a_terminal_with_the_given_environment() {
    let command = shell("test -t 0 && test -t 1 && echo tty; echo \"$TERM $GREETING\"")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GREETING", "hello");

    let mut session = Session::spawn(&command).unwrap();
    let mut out = String::new();
    assert!(read_until_closed(
        &session,
        &mut out,
        Instant::now() + PATIENCE
    ));
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));
    assert_eq!(plain_lines(&out), ["tty", "xterm-256color hello"]);
}

#[test]
fn typed_input_reaches_the_program() {
    let mut session = Session::spawn(&shell(
        "printf 'Proceed? [Y/n] '; read answer; echo got $answer",
    ))
    .unwrap();

    let mut out = String::new();
    let deadline = Instant::now() + PATIENCE;

    read_while(&session, &mut out, deadline, |out| {
        !out.contains("Proceed?")
    });

    session.input("y\n").unwrap();

    assert!(read_until_closed(
        &session,
        &mut out,
        Instant::now() + PATIENCE
    ));
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));
    assert_eq!(plain_lines(&out), ["Proceed? [Y/n] y", "got y"]);
}

#[test]
fn a_cancelled_program_dies_by_its_signal() {
    let mut session = Session::spawn(&PtyCommand::new("sleep").arg("30")).unwrap();
    let started = Instant::now();

    session.cancel(started).unwrap();
    assert!(session.is_cancelled());

    let exit = wait_exit(&mut session, started + Duration::from_secs(5)).unwrap();
    assert_eq!(exit, Some(Exit::Signal(2)), "SIGINT ended it");
}

#[test]
fn a_cancel_reaches_the_programs_children() {
    // The child ignores SIGINT and the SIGHUP its dying session leader
    // causes, so only a signal to the whole group stops it.
    let mut session =
        Session::spawn(&shell("(trap '' INT HUP; exec sleep 30) & echo $!; wait")).unwrap();

    let mut out = String::new();
    let deadline = Instant::now() + PATIENCE;

    read_while(&session, &mut out, deadline, |out| {
        plain_lines(out).is_empty()
    });

    let child = plain_lines(&out)[0].trim().to_owned();
    let proc_entry = Path::new("/proc").join(&child);
    assert!(proc_entry.exists(), "sleep {child} runs");

    session.cancel(Instant::now()).unwrap();
    assert!(wait_exit(&mut session, deadline).unwrap().is_some());

    while is_alive(&proc_entry) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        !is_alive(&proc_entry),
        "the background sleep {child} was stopped"
    );
}

/// A zombie has ended and only waits for its new parent to reap it.
fn is_alive(proc_entry: &Path) -> bool {
    std::fs::read_to_string(proc_entry.join("stat")).is_ok_and(|stat| {
        stat.rsplit_once(')')
            .is_some_and(|(_, rest)| !rest.trim_start().starts_with('Z'))
    })
}

#[test]
fn a_resize_reaches_the_program() {
    let command = shell("stty size; read go; stty size").size(TerminalSize {
        columns: 100,
        rows: 30,
    });

    let mut session = Session::spawn(&command).unwrap();
    let mut out = String::new();
    let deadline = Instant::now() + PATIENCE;

    read_while(&session, &mut out, deadline, |out| {
        plain_lines(out).is_empty()
    });

    session
        .resize(TerminalSize {
            columns: 120,
            rows: 40,
        })
        .unwrap();
    session.input("\n").unwrap();

    assert!(read_until_closed(&session, &mut out, deadline));
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));
    assert_eq!(plain_lines(&out), ["30 100", "", "40 120"]);
}

/// The caller also polls cancel and the exit, so a program that never
/// reads its terminal must not hold it. Whole lines fill the input
/// buffer, a canonical terminal drops an unfinished one. Past the queue
/// the input is refused instead of waiting.
#[test]
fn input_to_a_program_that_never_reads_never_holds_the_caller() {
    let mut session = Session::spawn(&PtyCommand::new("sleep").arg("30")).unwrap();
    let line = "x\n".repeat(32 * 1024);
    let started = Instant::now();

    let refused = (0_u8..100_u8).find_map(|_| session.input(&line).err());

    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the input held the caller"
    );
    assert!(
        refused.is_some_and(|error| error.kind() == io::ErrorKind::WouldBlock),
        "too much waiting input was taken"
    );

    session.cancel(Instant::now()).unwrap();

    assert_eq!(
        wait_exit(&mut session, started + PATIENCE).unwrap(),
        Some(Exit::Signal(2))
    );
    assert!(
        session
            .input("late\n")
            .is_err_and(|error| error.kind() == io::ErrorKind::BrokenPipe),
        "input after a cancel was taken"
    );
}

/// The output queue is bounded: a caller that stops reading stops the
/// program at its next write, it never fills the memory.
#[test]
fn a_caller_that_stops_reading_throttles_the_program() {
    let folder = tempfile::tempdir().unwrap();
    let done = folder.path().join("done");
    let mut session = Session::spawn(&shell(&format!(
        "head -c 4000000 /dev/zero | tr '\\0' x; touch {}",
        done.display()
    )))
    .unwrap();

    thread::sleep(Duration::from_secs(1));

    assert!(!done.exists(), "4 MB went out with nobody reading them");

    let mut out = String::new();
    assert!(read_until_closed(
        &session,
        &mut out,
        Instant::now() + PATIENCE
    ));
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));
    assert!(done.exists());
    assert!(out.len() >= 4_000_000);
}

#[test]
fn a_missing_program_names_the_failing_stage() {
    let error = Session::spawn(&PtyCommand::new("/nonexistent/program")).unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("start /nonexistent/program: "),
        "{error}"
    );
}

#[test]
fn a_finished_session_keeps_its_exit() {
    let mut session = Session::spawn(&shell("exit 0")).unwrap();

    assert!(session.id() > 0);
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));
    assert_eq!(session.try_wait().unwrap(), Some(Exit::Code(0)));
}

fn ctrl_c_and_quit(status: &str) -> Option<u64> {
    let line = status
        .lines()
        .find_map(|line| line.strip_prefix("SigIgn:"))?;

    let ignored = u64::from_str_radix(line.trim(), 16).ok()?;

    Some(ignored & ((1_u64 << (libc::SIGINT - 1_i32)) | (1_u64 << (libc::SIGQUIT - 1_i32))))
}

/// An app started with `&` from a shell ignores SIGINT and SIGQUIT, and
/// every child inherits that. The test binary runs itself in that state.
#[test]
fn a_program_takes_ctrl_c_even_when_the_app_ignores_it() {
    let copy = std::process::Command::new("env")
        .arg("--ignore-signal=INT,QUIT")
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "the_program_of_an_app_ignoring_ctrl_c",
            "--include-ignored",
        ])
        .env(IGNORING, "1")
        .output()
        .unwrap();

    // libtest also succeeds when the name matches no test.
    let report = String::from_utf8_lossy(&copy.stdout);
    assert!(copy.status.success(), "{report}");
    assert!(report.contains("1 passed"), "{report}");
}

#[test]
#[ignore = "run by a_program_takes_ctrl_c_even_when_the_app_ignores_it"]
fn the_program_of_an_app_ignoring_ctrl_c() {
    assert!(std::env::var_os(IGNORING).is_some());
    let own = std::fs::read_to_string("/proc/self/status").unwrap();
    assert_ne!(
        ctrl_c_and_quit(&own).unwrap(),
        0,
        "the copy should start ignoring"
    );

    let mut session = Session::spawn(&shell("cat /proc/self/status")).unwrap();
    let mut out = String::new();
    assert!(read_until_closed(
        &session,
        &mut out,
        Instant::now() + PATIENCE
    ));
    assert_eq!(finished(&mut session), Some(Exit::Code(0)));

    let child = plain_lines(&out).join("\n");
    assert_eq!(
        ctrl_c_and_quit(&child).unwrap(),
        0,
        "ignored signals in {child}"
    );
}
