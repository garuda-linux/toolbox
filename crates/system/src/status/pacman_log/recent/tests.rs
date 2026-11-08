use std::fs;
use std::io::Cursor;

use super::{RecentLogSource as _, tail_text};
use crate::ProbeError;
use crate::status::PacmanLog;
use crate::status::pacman_log::tests::LOG;

#[test]
fn a_short_log_is_read_whole() {
    let window = u64::try_from(LOG.len()).unwrap();

    assert_eq!(tail_text(&mut Cursor::new(LOG), window).unwrap(), LOG);
}

/// The window ends five bytes into the line before the last.
#[test]
fn a_long_log_keeps_its_newest_whole_lines() {
    let last = LOG.lines().last().unwrap();
    let window = u64::try_from(last.len() + 1 + 5).unwrap();

    assert_eq!(
        tail_text(&mut Cursor::new(LOG), window).unwrap(),
        format!("{last}\n")
    );
    assert_eq!(
        last,
        "[2026-11-08T17:36:03+0100] [ALPM] running 'zzz_post.hook'..."
    );
}

/// A made-up broken scriptlet line in front of the Lima log.
#[test]
fn a_bad_byte_does_not_hide_the_lines_around_it() {
    let mut log = LOG.as_bytes().to_vec();
    log.splice(0..0, *b"\xff scriptlet\n");

    let tail = tail_text(&mut Cursor::new(log), 1024 * 1024).unwrap();

    assert!(tail.contains("[ALPM] removed htop (3.5.2-1)"));
    assert!(tail.contains("Running 'pacman -Syu --noconfirm'"));
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
