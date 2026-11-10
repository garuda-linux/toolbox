mod steps;
mod writable;

use std::error::Error as _;
use std::fs;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _, symlink};

use tempfile::TempDir;
use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{ConfigWrite, FetchRefusal, Fingerprint, StepId};

use super::{WriteError, execute, write_config_file, write_real};
use crate::builtin::FileChange;
use crate::config_file::tests::{BACKUPS, path, text};
use crate::test_support::printed;

/// A root folder with `etc/fstab` (mode 0640).
fn system() -> TempDir {
    let root = tempfile::tempdir().unwrap();

    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/fstab"), "old = 1\n").unwrap();
    fs::set_permissions(
        root.path().join("etc/fstab"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    root
}

#[test]
fn a_write_keeps_owner_and_mode_and_backs_the_old_version_up() {
    let root = system();
    let target = root.path().join("etc/fstab");
    let before = fs::metadata(&target).unwrap();

    assert_eq!(
        write_config_file(
            root.path(),
            &path("/etc/fstab"),
            &text("new = 2\n"),
            Fingerprint::of(b"old = 1\n"),
        )
        .unwrap(),
        FileChange::Written
    );

    let after = fs::metadata(&target).unwrap();
    let backup = root.path().join(BACKUPS).join("etc/fstab");

    assert_eq!(fs::read_to_string(&target).unwrap(), "new = 2\n");
    assert_eq!(after.permissions().mode() & 0o7777, 0o640);
    assert_eq!((after.uid(), after.gid()), (before.uid(), before.gid()));
    assert_ne!(after.ino(), before.ino(), "the file was not replaced whole");
    assert_eq!(fs::read_to_string(&backup).unwrap(), "old = 1\n");
    assert_eq!(
        fs::metadata(&backup).unwrap().permissions().mode() & 0o7777,
        0o600
    );

    let left: Vec<_> = fs::read_dir(root.path().join("etc"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().contains(".tmp-"))
        .collect();

    assert!(left.is_empty(), "temporary files stayed behind");
}

#[test]
fn a_second_write_replaces_the_backup() {
    let root = system();
    let file = path("/etc/fstab");

    write_config_file(
        root.path(),
        &file,
        &text("two\n"),
        Fingerprint::of(b"old = 1\n"),
    )
    .unwrap();
    write_config_file(
        root.path(),
        &file,
        &text("three\n"),
        Fingerprint::of(b"two\n"),
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(root.path().join("etc/fstab")).unwrap(),
        "three\n"
    );
    assert_eq!(
        fs::read_to_string(root.path().join(BACKUPS).join("etc/fstab")).unwrap(),
        "two\n"
    );
}

#[test]
fn a_file_that_changed_meanwhile_is_not_overwritten() {
    let root = system();
    let target = root.path().join("etc/fstab");

    fs::write(&target, "someone else = 9\n").unwrap();

    let result = write_config_file(
        root.path(),
        &path("/etc/fstab"),
        &text("mine = 2\n"),
        Fingerprint::of(b"old = 1\n"),
    );

    assert!(matches!(result, Err(WriteError::Changed)));
    assert_eq!(fs::read_to_string(&target).unwrap(), "someone else = 9\n");
    assert!(
        !root.path().join(BACKUPS).exists(),
        "a backup without a write"
    );
}

#[test]
fn the_same_text_writes_nothing() {
    let root = system();

    assert_eq!(
        write_config_file(
            root.path(),
            &path("/etc/fstab"),
            &text("old = 1\n"),
            Fingerprint::of(b"old = 1\n"),
        )
        .unwrap(),
        FileChange::Unchanged
    );
    assert!(!root.path().join(BACKUPS).exists());
}

#[test]
fn the_write_of_a_real_path_never_follows_a_symlink() {
    let root = system();
    let victim = root.path().join("etc/victim");

    fs::write(&victim, "old = 1\n").unwrap();
    fs::remove_file(root.path().join("etc/fstab")).unwrap();
    symlink("victim", root.path().join("etc/fstab")).unwrap();

    let result = write_real(
        root.path(),
        &path("/etc/fstab"),
        &text("evil\n"),
        Fingerprint::of(b"old = 1\n"),
    );

    assert!(matches!(
        result,
        Err(WriteError::Refused(FetchRefusal::Link))
    ));
    assert_eq!(fs::read_to_string(&victim).unwrap(), "old = 1\n");
}

#[test]
fn a_symlinked_backup_folder_is_refused_and_the_file_stays() {
    let root = system();
    let elsewhere = root.path().join("elsewhere");

    fs::create_dir_all(&elsewhere).unwrap();
    fs::create_dir_all(root.path().join("var/lib")).unwrap();
    symlink(&elsewhere, root.path().join("var/lib/garuda-toolbox")).unwrap();

    let result = write_config_file(
        root.path(),
        &path("/etc/fstab"),
        &text("new\n"),
        Fingerprint::of(b"old = 1\n"),
    );

    assert!(matches!(result, Err(WriteError::Failed(_))));
    assert_eq!(
        fs::read_to_string(root.path().join("etc/fstab")).unwrap(),
        "old = 1\n"
    );
    assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
}

#[test]
fn a_missing_file_is_not_created() {
    let root = system();
    let result = write_config_file(
        root.path(),
        &path("/etc/new.conf"),
        &text("new\n"),
        Fingerprint::of(b""),
    );

    assert!(matches!(
        result,
        Err(WriteError::Refused(FetchRefusal::Missing))
    ));
    assert!(!root.path().join("etc/new.conf").exists());
}

#[test]
fn the_step_says_what_happened() {
    let root = system();
    let file = path("/etc/fstab");
    let mut events = Vec::new();

    assert_eq!(
        execute(
            root.path(),
            StepId::new(1),
            &ConfigWrite {
                path: file.clone(),
                content: text("new\n"),
                original: Fingerprint::of(b"old = 1\n"),
            },
            &mut events,
        ),
        Outcome::Succeeded
    );
    assert_eq!(
        printed(&events),
        "wrote /etc/fstab, the previous version is /var/lib/garuda-toolbox/backups/etc/fstab\r\n"
    );

    let mut stale = Vec::new();

    assert_eq!(
        execute(
            root.path(),
            StepId::new(2),
            &ConfigWrite {
                path: file,
                content: text("newer\n"),
                original: Fingerprint::of(b"old = 1\n"),
            },
            &mut stale,
        ),
        Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: Vec::new()
        }
    );
    assert!(printed(&stale).contains("changed since it was opened"));
    assert_eq!(
        fs::read_to_string(root.path().join("etc/fstab")).unwrap(),
        "new\n"
    );
}

#[test]
fn a_write_error_says_why_in_the_words_of_the_console() {
    let failed = WriteError::Failed(std::io::Error::other("read-only"));

    assert_eq!(
        WriteError::Refused(FetchRefusal::NotText).to_string(),
        "the file is no text"
    );
    assert_eq!(
        WriteError::Changed.to_string(),
        "the file changed since it was opened"
    );
    assert!(WriteError::Changed.source().is_none());
    assert_eq!(failed.to_string(), "read-only");
    assert_eq!(failed.source().unwrap().to_string(), "read-only");
}
