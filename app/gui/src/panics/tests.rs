use std::panic::Location;

use super::{MAX_CRASH_BYTES, headline, record};

#[test]
fn a_panic_is_named_with_its_thread_place_and_message() {
    let here = Location::caller();
    let line = headline(
        Some("toolbox-worker"),
        Some(here),
        Some("index out of bounds"),
    );

    assert_eq!(
        line,
        format!(
            "panic in toolbox-worker at {}:{}:{}: index out of bounds",
            here.file(),
            here.line(),
            here.column()
        )
    );
}

#[test]
fn what_a_panic_does_not_say_is_left_out() {
    assert_eq!(
        headline(None, None, None),
        "panic in a thread without a name: no message"
    );
}

#[test]
fn every_panic_is_added_to_the_crash_file_for_the_owner_alone() {
    use std::os::unix::fs::PermissionsExt as _;

    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("state/garuda-toolbox/crash.log");

    record(&file, "panic in main: first\n").unwrap();
    record(&file, "panic in main: second\n").unwrap();

    let text = std::fs::read_to_string(&file).unwrap();
    let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;

    assert_eq!(text, "panic in main: first\npanic in main: second\n");
    assert_eq!(mode, 0o600, "the backtraces hold paths of the user");
}

#[test]
fn a_full_crash_file_moves_aside_once_and_starts_over() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("crash.log");
    let older = folder.path().join("crash.log.1");
    let full = "x".repeat(usize::try_from(MAX_CRASH_BYTES).unwrap() - 4);

    std::fs::write(&older, "the oldest crashes\n").unwrap();
    std::fs::write(&file, &full).unwrap();
    record(&file, "abc\n").unwrap();
    assert!(
        std::fs::read_to_string(&older).unwrap() == "the oldest crashes\n",
        "a text that still fits moved the file"
    );

    record(&file, "panic in main: late\n").unwrap();

    assert_eq!(
        std::fs::read_to_string(&older).unwrap(),
        format!("{full}abc\n")
    );
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "panic in main: late\n"
    );
}

#[test]
fn a_crash_file_that_cannot_be_written_says_so() {
    let folder = tempfile::tempdir().unwrap();
    let blocker = folder.path().join("not-a-folder");
    std::fs::write(&blocker, "").unwrap();

    assert!(record(&blocker.join("crash.log"), "panic in main\n").is_err());
}

#[test]
fn a_crash_file_that_is_a_link_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let target = folder.path().join("elsewhere");
    let file = folder.path().join("crash.log");

    std::fs::write(&target, "").unwrap();
    std::os::unix::fs::symlink(&target, &file).unwrap();

    record(&file, "panic in main: boom\n").unwrap_err();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "");
}
