use std::fs;

use log::LevelFilter;
use parking_lot::Mutex;

use crate::file::FileLogger;
use crate::{LOGGER, open_file};

/// The global level is process state: tests that touch it hold this lock
/// and put it back.
static GUARD: Mutex<()> = Mutex::new(());

#[test]
fn starting_and_opening_writes_the_version_line_in_the_given_folder() {
    let dir = tempfile::tempdir().unwrap();
    let guard = GUARD.lock();

    let old_level = log::max_level();

    LOGGER.get_or_init(FileLogger::buffered);
    log::set_max_level(LevelFilter::Debug);

    let folder = dir.path().join("garuda-toolbox");
    let path = open_file(Some(&folder), "garuda-toolbox-test").unwrap();
    let text = fs::read_to_string(&path).unwrap();

    assert_eq!(path, folder.join("garuda-toolbox-test.log"));
    assert!(text.contains("started (log level debug)"), "{text}");

    log::set_max_level(old_level);
    drop(guard);
}

#[test]
fn the_version_line_is_written_whatever_the_level() {
    let dir = tempfile::tempdir().unwrap();
    let guard = GUARD.lock();

    let old_level = log::max_level();

    LOGGER.get_or_init(FileLogger::buffered);
    log::set_max_level(LevelFilter::Warn);

    let path = open_file(Some(dir.path()), "garuda-toolbox-quiet").unwrap();
    let text = fs::read_to_string(&path).unwrap();

    assert!(text.contains("started (log level warn)"), "{text}");

    log::set_max_level(old_level);
    drop(guard);
}
