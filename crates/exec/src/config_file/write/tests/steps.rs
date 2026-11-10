use std::fs;

use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{ConfigFilePath, ConfigWrite, FetchRefusal, Fingerprint, StepId};

use super::super::{ReadError, WriteError, execute, refusal_text};
use super::text;
use crate::test_support::printed;

fn write(root: &std::path::Path, name: &str) -> (Outcome, String) {
    let mut events = Vec::new();
    let outcome = execute(
        root,
        StepId::new(1),
        &ConfigWrite {
            path: ConfigFilePath::new(format!("/etc/{name}")).unwrap(),
            content: text("new = 2\n"),
            original: Fingerprint::of(b"old = 1\n"),
        },
        &mut events,
    );

    (outcome, printed(&events))
}

#[test]
fn an_unchanged_file_skips_its_step() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/fstab"), "new = 2\n").unwrap();
    let mut events = Vec::new();
    let outcome = execute(
        root.path(),
        StepId::new(1),
        &ConfigWrite {
            path: ConfigFilePath::new("/etc/fstab").unwrap(),
            content: text("new = 2\n"),
            original: Fingerprint::of(b"new = 2\n"),
        },
        &mut events,
    );

    assert_eq!(outcome, Outcome::Skipped);
}

#[test]
fn a_changed_file_is_not_overwritten() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/fstab"), "meanwhile = 3\n").unwrap();

    let (outcome, text) = write(root.path(), "fstab");

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: Vec::new()
        }
    );
    assert!(text.contains("changed since it was opened"), "{text}");
}

#[test]
fn refusals_say_why_in_plain_words() {
    assert_eq!(
        refusal_text(FetchRefusal::Missing),
        "the file does not exist"
    );
    assert_eq!(
        refusal_text(FetchRefusal::Link),
        "a link stands in its place since it was looked at"
    );
    assert_eq!(
        refusal_text(FetchRefusal::Dangling),
        "it is a link that leads to no file"
    );
    assert_eq!(
        refusal_text(FetchRefusal::Outside),
        "it is a link to a file outside the folders the toolbox edits"
    );
    assert_eq!(
        refusal_text(FetchRefusal::NotRegular),
        "it is not a regular file"
    );
    assert_eq!(
        refusal_text(FetchRefusal::TooLarge),
        "the file is too large"
    );
    assert_eq!(refusal_text(FetchRefusal::NotText), "the file is no text");
}

#[test]
fn a_failed_read_becomes_a_failed_write() {
    let failed: WriteError = ReadError::Failed(std::io::Error::other("gone")).into();

    assert_eq!(failed.to_string(), "gone");
}
