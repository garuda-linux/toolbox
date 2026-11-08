use super::*;

impl FileLogger {
    fn open(path: PathBuf) -> Self {
        Self::open_retrying(path, RETRY_AFTER)
    }

    fn open_retrying(path: PathBuf, retry_after: Duration) -> Self {
        let mut state = State::empty();

        state.reopen(&path);
        state.path = Some(path);

        Self {
            retry_after,
            state: Mutex::new(state),
            quiet: AtomicBool::new(false),
        }
    }
}

fn size_of(path: &Path) -> u64 {
    fs::metadata(path).map_or(0, |meta| meta.len())
}

fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;

    fs::metadata(path).map_or(0, |meta| meta.permissions().mode() & 0o777)
}

#[test]
fn the_log_file_is_the_owners_alone_to_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state/garuda-toolbox/app.log");

    FileLogger::open(path.clone()).write_line("a line\n");
    assert_eq!(mode_of(&path), 0o600);

    let rotated = dir.path().join("state/garuda-toolbox/app.log.1");

    fs::write(&path, "x".repeat(usize::try_from(MAX_SIZE).unwrap())).unwrap();
    FileLogger::open(path).write_line("rotate\n");

    assert!(rotated.exists());
    assert_eq!(mode_of(&rotated), 0o600);
}

#[test]
fn a_full_file_is_rotated_before_the_next_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state/garuda-toolbox/app.log");
    let rotated = dir.path().join("state/garuda-toolbox/app.log.1");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // One byte short of the limit, so the size taken from the existing file
    // decides when to rotate.
    let almost_full = usize::try_from(MAX_SIZE - 1).unwrap();
    fs::write(&path, "x".repeat(almost_full)).unwrap();

    let logger = FileLogger::open(path.clone());
    logger.write_line("first\n");
    assert!(
        !rotated.exists(),
        "rotation waits until the limit is reached"
    );
    assert_eq!(size_of(&path), MAX_SIZE + 5);

    logger.write_line("second\n");
    assert_eq!(size_of(&rotated), MAX_SIZE + 5);
    assert_eq!(fs::read_to_string(&path).unwrap(), "second\n");
}

#[test]
fn lines_are_appended_to_an_existing_log() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("new/folder/app.log");

    FileLogger::open(path.clone()).write_line("one\n");
    FileLogger::open(path.clone()).write_line("two\n");

    assert_eq!(fs::read_to_string(&path).unwrap(), "one\ntwo\n");
}

#[test]
fn an_unwritable_log_drops_lines_without_failing() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    fs::write(&blocker, "").unwrap();

    let logger = FileLogger::open(blocker.join("app.log"));
    logger.write_line("lost\n");

    assert_eq!(fs::read_to_string(&blocker).unwrap(), "");
}

#[test]
fn a_log_that_could_not_be_opened_is_tried_again_on_a_later_line() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("state");
    let path = blocker.join("app.log");
    fs::write(&blocker, "").unwrap();

    let logger = FileLogger::open_retrying(path.clone(), Duration::ZERO);
    logger.write_line("lost\n");

    fs::remove_file(&blocker).unwrap();
    logger.write_line("kept\n");
    logger.write_line("too\n");

    assert_eq!(fs::read_to_string(&path).unwrap(), "kept\ntoo\n");
}

#[test]
fn a_failed_open_is_not_tried_again_before_its_pause_is_over() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("state");
    let path = blocker.join("app.log");
    fs::write(&blocker, "").unwrap();

    let logger = FileLogger::open_retrying(path.clone(), Duration::from_secs(3600));

    fs::remove_file(&blocker).unwrap();
    logger.write_line("lost\n");

    assert!(!path.exists(), "the open was tried for every line");
}

#[test]
fn lines_before_the_file_are_buffered_until_it_opens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("garuda-toolbox/app.log");
    let logger = FileLogger::buffered();

    logger.write_line("early one\n");
    logger.write_line("early two\n");
    assert!(!path.exists(), "the file opened before the claim");

    logger.open_file(&path);
    logger.write_line("late\n");

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "early one\nearly two\nlate\n"
    );
}

/// Without a safe folder no file ever opens, so holding lines for one
/// would only cost memory.
#[test]
fn stderr_only_drops_the_buffer_and_keeps_nothing_after() {
    let logger = FileLogger::buffered();

    logger.write_line("early\n");
    logger.stderr_only();
    logger.write_line("late\n");

    assert!(logger.state.lock().buffered.is_empty());
}

#[test]
fn a_full_buffer_keeps_the_latest_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");
    let logger = FileLogger::buffered();

    for index in 0..(BUFFERED_LINES + 10) {
        logger.write_line(&format!("line {index}\n"));
    }

    logger.open_file(&path);
    let text = fs::read_to_string(&path).unwrap();

    assert!(
        !text.contains("line 0\n"),
        "the oldest line survived a full buffer"
    );
    assert!(text.contains(&format!("line {}\n", BUFFERED_LINES + 9)));
}

#[test]
fn a_line_logged_while_the_file_opens_never_deadlocks() {
    use std::sync::{Arc, mpsc};
    use std::thread;

    let dir = tempfile::tempdir().unwrap();
    let (done, finished) = mpsc::channel();

    for round in 0..200_u32 {
        let logger = Arc::new(FileLogger::buffered());
        let writer = Arc::clone(&logger);
        let path = dir.path().join(format!("{round}.log"));
        let done = done.clone();

        thread::Builder::new()
            .name(format!("opener-{round}"))
            .spawn(move || {
                let lines = thread::Builder::new()
                    .name(format!("writer-{round}"))
                    .spawn(move || {
                        for _ in 0..50_u32 {
                            writer.write_line("a line\n");
                        }
                    })
                    .unwrap();

                logger.open_file(&path);
                lines.join().ok();
                done.send(()).ok();
            })
            .unwrap();
    }

    for _ in 0..200_u32 {
        finished
            .recv_timeout(Duration::from_secs(10))
            .expect("a log call and open_file held each other's lock");
    }
}

#[test]
fn an_unmakable_folder_is_no_sink() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    fs::write(&blocker, "").unwrap();

    assert!(open_sink(&blocker.join("app.log")).is_none());
}

#[test]
fn a_rotation_failure_is_reported_not_lost() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");

    // Non-empty folders nowhere renames onto: every rename fails instead
    // of moving history over history.
    for number in 1..=3_u32 {
        let numbered = dir.path().join(format!("app.log.{number}"));
        fs::create_dir_all(&numbered).unwrap();
        fs::write(numbered.join("kept"), "").unwrap();
    }

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

#[test]
fn a_quiet_terminal_keeps_the_lines_in_the_file_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");
    let logger = FileLogger::open(path.clone());

    assert!(logger.echoes());
    logger.quiet_terminal();
    logger.emit(log::Level::Info, "cli", "answer printed");

    assert!(!logger.echoes());
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("cli: answer printed"),
        "the line did not reach the file"
    );
}
