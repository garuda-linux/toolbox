mod tail;

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{PacmanLog, UpdateLogSource as _, last_completed};
use crate::ProbeError;

const LOG: &str = "\
[2026-09-01T10:00:00+0200] [PACMAN] Running 'pacman -Syu'
[2026-09-01T10:00:05+0200] [ALPM] transaction started
[2026-09-01T10:00:09+0200] [ALPM] upgraded bash (5.2-1 -> 5.3-1)
[2026-09-01T10:00:10+0200] [ALPM] transaction completed
[2026-09-01T10:00:11+0200] [ALPM] running '30-systemd-update.hook'...
[2026-10-09T14:03:00+0200] [PACMAN] Running 'pacman -S vim'
[2026-10-09T14:03:11+0200] [ALPM] transaction completed
[2026-10-09T14:03:12+0200] [ALPM] running 'texinfo-install.hook'...
[2026-10-09T15:00:00+0200] [PACMAN] Running 'pacman -Syu'
[2026-10-09T15:00:01+0200] [ALPM] transaction started
[2026-10-09T15:00:02+0200] [ALPM] transaction failed
";

fn at(seconds: u64) -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

fn parse(log: &[u8]) -> Option<SystemTime> {
    last_completed(log).unwrap()
}

#[test]
fn the_last_completed_transaction_counts() {
    assert_eq!(parse(LOG.as_bytes()), at(1_791_547_391));
}

#[test]
fn a_log_without_a_completed_transaction_has_none() {
    let log = "[2026-10-09T15:00:01+0200] [ALPM] transaction started\n\
               [2026-10-09T15:00:02+0200] [ALPM] transaction failed\n";

    assert_eq!(parse(log.as_bytes()), None);
    assert_eq!(parse(b""), None);
}

#[test]
fn the_old_timestamp_format_is_read() {
    let log = "[2017-12-01 09:30] [ALPM] transaction started\n\
               [2017-12-01 09:31] [ALPM] transaction completed\n";

    assert_eq!(parse(log.as_bytes()), at(1_512_120_660));
}

#[test]
fn old_and_new_formats_mix_in_one_log() {
    let log = "[2017-12-01 09:31] [ALPM] transaction completed\n\
               [2024-01-02T10:11:00+0000] [ALPM] transaction completed\n";

    assert_eq!(parse(log.as_bytes()), at(1_704_190_260));
}

/// Scriptlets print whatever they like, a broken byte must not hide the
/// lines after it.
#[test]
fn invalid_utf8_and_odd_lines_are_skipped() {
    let mut log = b"[2024-01-02T10:11:00+0000] [ALPM] transaction completed\n".to_vec();
    log.extend_from_slice(b"[2024-01-03T00:00:00+0000] [ALPM-SCRIPTLET] \xff\xfe broken\n");
    log.extend_from_slice(b"no timestamp [ALPM] transaction completed\n");
    log.extend_from_slice(b"[not a date] [ALPM] transaction completed\n");
    log.extend_from_slice(b"[2024-01-02T10:11:00+0000] [PACMAN] transaction completed\n");
    log.extend_from_slice(b"\n");

    assert_eq!(parse(&log), at(1_704_190_260));
}

#[test]
fn a_last_line_without_newline_counts() {
    let log = "[2024-01-02T10:11:00+0000] [ALPM] transaction completed";

    assert_eq!(parse(log.as_bytes()), at(1_704_190_260));
}

#[test]
fn the_adapter_reads_the_given_file() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("pacman.log");
    fs::write(&path, LOG).unwrap();

    assert_eq!(PacmanLog::new(path).last_update(), Ok(at(1_791_547_391)));
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
