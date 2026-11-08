mod tail;

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{PacmanLog, UpdateLogSource as _, last_completed};
use crate::ProbeError;

/// `/var/log/pacman.log` of a Lima session: an upgrade, installs, a
/// reinstall, a downgrade and a removal.
pub(super) const LOG: &str = include_str!("fixtures/pacman.log");
/// The last 1500 lines of the log of an installed Garuda. They start in
/// the middle of a transaction.
const INSTALLED: &str = include_str!("fixtures/installed-pacman.log");

/// `2026-11-08T17:36:00+0100`, the last `transaction completed` in [`LOG`].
pub(super) const LAST: u64 = 1_794_155_760;

fn at(seconds: u64) -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

fn parse(log: &[u8]) -> Option<SystemTime> {
    last_completed(log).unwrap()
}

/// The lines of `log` before its first `transaction completed`.
fn before_first_completed(log: &str) -> &str {
    let completed = log.find("] [ALPM] transaction completed").unwrap();

    let last_break = log.get(..completed).unwrap().rfind('\n').unwrap();

    log.get(..=last_break).unwrap()
}

#[test]
fn the_last_completed_transaction_counts() {
    assert_eq!(parse(LOG.as_bytes()), at(LAST));
    assert_eq!(parse(INSTALLED.as_bytes()), at(1_794_055_708));
}

/// The installed log cut before its first completed transaction, and the
/// Lima log cut while its first transaction runs.
#[test]
fn a_log_without_a_completed_transaction_has_none() {
    assert_eq!(parse(before_first_completed(INSTALLED).as_bytes()), None);
    assert_eq!(parse(before_first_completed(LOG).as_bytes()), None);
    assert_eq!(parse(b""), None);
}

/// The Lima log with its last completed transaction written in the format
/// of pacman before 5.1.
#[test]
fn the_old_timestamp_format_is_read() {
    let log = LOG.replacen(
        "[2026-11-08T17:36:00+0100] [ALPM] transaction completed",
        "[2026-11-08 17:36] [ALPM] transaction completed",
        1,
    );

    assert_eq!(parse(log.as_bytes()), at(1_794_159_360));
}

/// The Lima log with its first completed transaction in the old format.
#[test]
fn old_and_new_formats_mix_in_one_log() {
    let log = LOG.replacen(
        "[2026-11-08T17:30:07+0100] [ALPM] transaction completed",
        "[2026-11-08 17:30] [ALPM] transaction completed",
        1,
    );

    assert_ne!(log, LOG);
    assert_eq!(parse(log.as_bytes()), at(LAST));
}

/// Scriptlets print whatever they like, a broken byte must not hide the
/// lines after it. The made-up lines go before the Lima log's last
/// completed transaction.
#[test]
fn invalid_utf8_and_odd_lines_are_skipped() {
    let completed = LOG.rfind("[2026-11-08T17:36:00+0100] [ALPM] transaction completed");
    let (head, tail) = LOG.split_at(completed.unwrap());
    let mut log = head.as_bytes().to_vec();
    log.extend_from_slice(b"[2026-11-08T17:36:00+0100] [ALPM-SCRIPTLET] \xff\xfe broken\n");
    log.extend_from_slice(b"\n");
    log.extend_from_slice(tail.as_bytes());
    log.extend_from_slice(b"no timestamp [ALPM] transaction completed\n");
    log.extend_from_slice(b"[not a date] [ALPM] transaction completed\n");
    log.extend_from_slice(b"[2026-11-09T10:11:00+0000] [PACMAN] transaction completed\n");

    assert_eq!(parse(&log), at(LAST));
}

/// The Lima log cut right after its last completed transaction.
#[test]
fn a_last_line_without_newline_counts() {
    let end = LOG.rfind("transaction completed").unwrap() + "transaction completed".len();

    assert_eq!(parse(&LOG.as_bytes()[..end]), at(LAST));
}

#[test]
fn the_adapter_reads_the_given_file() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("pacman.log");
    fs::write(&path, LOG).unwrap();

    assert_eq!(PacmanLog::new(path).last_update(), Ok(at(LAST)));
}

#[test]
fn a_missing_log_is_unavailable() {
    let log = PacmanLog::new(Path::new("/nonexistent/toolbox/pacman.log"));

    assert_eq!(log.last_update(), Err(ProbeError::Unavailable));
}

/// Root reads every file, so the check only means something for a user.
#[test]
fn an_unreadable_log_is_unavailable() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("pacman.log");
    fs::write(&path, LOG).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

    if fs::read(&path).is_ok() {
        return;
    }

    assert_eq!(
        PacmanLog::new(path).last_update(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn a_folder_instead_of_a_log_fails() {
    let folder = tempfile::tempdir().unwrap();

    assert!(matches!(
        PacmanLog::new(folder.path()).last_update(),
        Err(ProbeError::Failed(_))
    ));
}
