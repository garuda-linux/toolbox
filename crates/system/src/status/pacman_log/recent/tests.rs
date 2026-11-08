use std::fs;
use std::io::Cursor;

use super::{RecentLogSource as _, tail_text};
use crate::ProbeError;
use crate::status::PacmanLog;

const LOG: &str = "\
[2026-09-01T10:00:09+0200] [ALPM] upgraded bash (5.2-1 -> 5.3-1)
[2026-10-09T14:03:05+0200] [ALPM] installed vim (9.1-1)
";

#[test]
fn a_short_log_is_read_whole() {
    assert_eq!(tail_text(&mut Cursor::new(LOG), 1024).unwrap(), LOG);
}

#[test]
fn a_long_log_keeps_its_newest_whole_lines() {
    let tail = tail_text(&mut Cursor::new(LOG), 70).unwrap();

    assert_eq!(
        tail,
        "[2026-10-09T14:03:05+0200] [ALPM] installed vim (9.1-1)\n"
    );
}

#[test]
fn a_bad_byte_does_not_hide_the_lines_around_it() {
    let mut log = LOG.as_bytes().to_vec();
    log.splice(0..0, *b"\xff scriptlet\n");

    let tail = tail_text(&mut Cursor::new(log), 4096).unwrap();

    assert!(tail.contains("installed vim"));
    assert!(tail.contains("upgraded bash"));
}

#[test]
fn the_log_file_is_read_and_a_missing_one_is_unavailable() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("pacman.log");
    fs::write(&path, LOG).unwrap();

    assert_eq!(PacmanLog::new(&path).recent(), Ok(LOG.to_owned()));
    assert_eq!(
        PacmanLog::new(folder.path().join("missing.log")).recent(),
        Err(ProbeError::Unavailable)
    );
}
