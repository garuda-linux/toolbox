use super::*;

/// Non-empty folders nowhere renames onto: every rename fails instead of
/// moving history over history.
fn block_rotation(folder: &Path) {
    for number in 1..=3_u32 {
        let numbered = folder.join(format!("app.log.{number}"));
        fs::create_dir_all(&numbered).unwrap();
        fs::write(numbered.join("kept"), "").unwrap();
    }
}

#[test]
fn a_rotation_failure_is_reported_not_lost() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");

    block_rotation(dir.path());

    rotate(&path);

    for number in 1..=3_u32 {
        assert!(
            dir.path().join(format!("app.log.{number}")).is_dir(),
            "history was renamed over history"
        );
    }

    fs::write(&path, "current").unwrap();
    rotate(&path);

    assert_eq!(fs::read_to_string(&path).unwrap(), "current");
}

/// `app.log.3` is a non-empty folder, so only the move of `app.log.2`
/// fails. The current file still moves and its path is free.
#[test]
fn a_failed_older_rename_still_frees_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");
    let oldest = dir.path().join("app.log.3");

    fs::create_dir_all(&oldest).unwrap();
    fs::write(oldest.join("kept"), "").unwrap();
    fs::write(dir.path().join("app.log.2"), "two").unwrap();
    fs::write(dir.path().join("app.log.1"), "one").unwrap();
    fs::write(&path, "current").unwrap();

    assert!(rotate(&path));
    assert!(!path.exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("app.log.1")).unwrap(),
        "current"
    );
}

/// The full file stays where it is, so a line during the pause is not
/// appended to it.
#[test]
fn a_failed_rotation_pauses_like_a_failed_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");

    fs::write(&path, "x".repeat(usize::try_from(MAX_SIZE).unwrap())).unwrap();
    block_rotation(dir.path());

    let logger = FileLogger::open(path.clone());

    logger.write_line("rotate\n");
    logger.write_line("paused\n");

    let state = logger.state.lock();
    assert!(state.sink.is_none());
    assert!(state.failed.is_some());
    drop(state);
    assert_eq!(size_of(&path), MAX_SIZE);
}

#[test]
fn rotation_keeps_a_bounded_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");

    for round in 0..5_u32 {
        fs::write(&path, format!("{round}")).unwrap();
        rotate(&path);
    }

    let read = |name: &str| fs::read_to_string(dir.path().join(name)).unwrap_or_default();
    assert_eq!(read("app.log.1"), "4");
    assert_eq!(read("app.log.3"), "2");
    assert!(!dir.path().join("app.log.4").exists());
    assert!(!path.exists());
}
