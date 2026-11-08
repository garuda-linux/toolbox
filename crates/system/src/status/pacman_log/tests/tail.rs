use std::cell::Cell;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::time::{Duration, SystemTime};

use super::super::last_in_tail;

const WINDOW: usize = 1024;
const COMPLETED: &str = "[2026-10-09T14:03:11+0000] [ALPM] transaction completed\n";
const NOISE: &str = "[2026-10-09T15:00:00+0000] [PACMAN] Running 'pacman -Sy'\n";

struct Counted<'bytes> {
    log: Cursor<Vec<u8>>,
    read: &'bytes Cell<usize>,
}

impl Read for Counted<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = self.log.read(buf)?;

        self.read.set(self.read.get() + count);
        Ok(count)
    }
}

impl Seek for Counted<'_> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.log.seek(pos)
    }
}

fn scan(log: &str) -> (Option<SystemTime>, usize) {
    let read = Cell::new(0);
    let mut log = Counted {
        log: Cursor::new(log.as_bytes().to_vec()),
        read: &read,
    };

    (
        last_in_tail(&mut log, u64::try_from(WINDOW).unwrap()).unwrap(),
        read.get(),
    )
}

fn stamp() -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(1_791_554_591))
}

#[test]
fn only_the_end_of_a_long_log_is_read() {
    let log = format!("{}{COMPLETED}{NOISE}", NOISE.repeat(10_000));
    let (last, read) = scan(&log);

    assert_eq!(last, stamp());
    assert!(read <= 2 * WINDOW, "{read} bytes read");
}

#[test]
fn a_transaction_before_the_end_is_still_found() {
    let log = format!("{NOISE}{COMPLETED}{}", NOISE.repeat(2_000));
    let (last, read) = scan(&log);

    assert_eq!(last, stamp());
    assert!(read < 3 * log.len(), "{read} bytes read");
}

#[test]
fn a_log_without_a_transaction_is_read_to_its_start_and_has_none() {
    assert_eq!(scan(&NOISE.repeat(500)).0, None);
    assert_eq!(scan("").0, None);
    assert_eq!(scan(COMPLETED).0, stamp());
}

/// A window that starts inside a line must not take its rest for a line:
/// here the rest reads like a transaction, and the log holds none.
#[test]
fn the_line_the_window_cuts_is_not_read() {
    let trap = "[2026-10-09T14:03:11+0000] [ALPM] upgraded x (1 -> [2030-01-01T00:00:00+0000] [ALPM] transaction completed\n";
    let rest = trap.len() - trap.find("[2030").unwrap() - 1;
    let fill = WINDOW - rest;
    let log = format!("{trap}{}\n", "x".repeat(fill - 1));

    assert_eq!(
        log.len() - WINDOW,
        trap.len() - rest,
        "the window starts a byte into the trap"
    );
    assert_eq!(scan(&log).0, None);
}

#[test]
fn a_window_that_starts_at_a_line_keeps_that_line() {
    let fill = WINDOW - COMPLETED.len();
    let log = format!("{NOISE}{COMPLETED}{}\n", "x".repeat(fill - 1));

    assert_eq!(log.len() - NOISE.len(), WINDOW);
    assert_eq!(scan(&log).0, stamp());
}
