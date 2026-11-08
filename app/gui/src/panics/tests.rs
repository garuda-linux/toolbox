use std::panic::Location;

use super::{headline, record};

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
fn a_crash_file_that_cannot_be_written_says_so() {
    let folder = tempfile::tempdir().unwrap();
    let blocker = folder.path().join("not-a-folder");
    std::fs::write(&blocker, "").unwrap();

    assert!(record(&blocker.join("crash.log"), "panic in main\n").is_err());
}
