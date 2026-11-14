use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::{ProbeError, Stop, from_io};

fn json() -> Arc<serde_json::Error> {
    Arc::new(serde_json::from_str::<u8>("x").unwrap_err())
}

#[test]
fn a_failure_reads_as_before_behind_its_prefix() {
    let timed_out = ProbeError::TimedOut {
        command_line: "sleep 60".to_owned(),
        after: Duration::from_millis(500),
    };

    assert_eq!(
        timed_out.to_string(),
        "probe failed: sleep 60: no answer within 0.5 s, stopped"
    );
    assert_eq!(
        ProbeError::NotAFile {
            path: PathBuf::from("/var/log/pacman.log")
        }
        .reason()
        .as_deref(),
        Some("/var/log/pacman.log is no file")
    );
    assert_eq!(
        ProbeError::Unavailable.to_string(),
        "not available on this system"
    );
    assert_eq!(ProbeError::Unavailable.reason(), None);
}

#[test]
fn a_stop_names_why() {
    let stopped = ProbeError::Stopped {
        command_line: "lsblk".to_owned(),
        why: Stop::TooLarge,
    };

    assert_eq!(
        stopped.reason().as_deref(),
        Some("lsblk: the program printed too much")
    );
}

#[test]
fn a_wrapped_error_stays_the_source() {
    let error = ProbeError::Json {
        what: "ip printed no route list".to_owned(),
        error: json(),
    };

    assert!(std::error::Error::source(&error).is_some());
    assert!(std::error::Error::source(&ProbeError::Panicked).is_none());
}

#[test]
fn failures_are_equal_when_of_one_kind_and_text() {
    let parse = |what: &str| ProbeError::Json {
        what: what.to_owned(),
        error: json(),
    };

    assert_eq!(parse("ip"), parse("ip"));
    assert_ne!(parse("ip"), parse("health"));
    assert_ne!(
        ProbeError::fake("ip: expected value at line 1 column 1"),
        parse("ip"),
        "the same words of another kind"
    );

    let denied = || from_io("x", io::Error::from(io::ErrorKind::PermissionDenied));
    assert_eq!(denied(), denied());
    assert_eq!(
        from_io("x", io::Error::from(io::ErrorKind::NotFound)),
        ProbeError::Unavailable
    );
}
