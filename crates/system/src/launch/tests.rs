use std::fs::File;
use std::io;
use std::path::Path;

use super::{ProgramRunner as _, RunError, SystemRunner};

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| (*word).to_owned()).collect()
}

#[tokio::test]
async fn a_missing_program_is_not_started() {
    let error = SystemRunner
        .run(Path::new("/nonexistent/garuda-installer"), &[], None)
        .await
        .unwrap_err();

    let kind = match error {
        RunError::NotStarted(error) => error.kind(),
        other @ RunError::Lost(_) => panic!("not started: {other:?}"),
    };

    assert_eq!(kind, io::ErrorKind::NotFound);
}

#[tokio::test]
async fn a_program_that_ends_well_reports_success() {
    let status = SystemRunner
        .run(Path::new("true"), &[], None)
        .await
        .unwrap();

    assert!(status.success());
}

#[tokio::test]
async fn a_program_that_fails_reports_its_code() {
    let status = SystemRunner
        .run(Path::new("sh"), &words(&["-c", "exit 3"]), None)
        .await
        .unwrap();

    assert_eq!(status.code(), Some(3_i32));
}

#[tokio::test]
async fn the_stderr_file_receives_what_the_program_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stderr");
    let file = File::create(&path).unwrap();

    SystemRunner
        .run(
            Path::new("sh"),
            &words(&["-c", "echo oops >&2"]),
            Some(file),
        )
        .await
        .unwrap();

    assert_eq!(std::fs::read_to_string(path).unwrap(), "oops\n");
}
