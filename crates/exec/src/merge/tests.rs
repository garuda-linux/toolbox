mod sudoers;

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;

use tempfile::TempDir;
use toolbox_plan::procedure::Command;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::testing::merge::{PACMAN_FILES, SUDO_FILES};
use toolbox_plan::{
    ConfigText, Fingerprint, MergeChoice, MergeCopy, MergeResolve, Resolution, StepId,
};

use super::{MergeError, Resolved, execute, is_backup, is_fetchable, resolve, snapshot};
use crate::config_file::WriteError;
use crate::test_support::{BACKUPS, assert_failed, path, printed, text};

const CURRENT: &str = "#Color\nParallelDownloads = 5\n";
const NEW: &str = "Color\nParallelDownloads = 5\n";

/// A root with pacman's and sudo's `files` lists as the Lima VM has
/// them, `/etc/pacman.conf` (mode 0640) and its `.pacnew`.
fn system() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    let local = root.path().join("var/lib/pacman/local");

    for (package, files) in [
        ("pacman-7.1.0.r9.g54d9411-4", PACMAN_FILES),
        ("sudo-1.9.17.p2-7", SUDO_FILES),
    ] {
        fs::create_dir_all(local.join(package)).unwrap();
        fs::write(local.join(package).join("files"), files).unwrap();
    }

    fs::write(local.join("ALPM_DB_VERSION"), "9\n").unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/pacman.conf"), CURRENT).unwrap();
    fs::set_permissions(
        root.path().join("etc/pacman.conf"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    fs::write(root.path().join("etc/pacman.conf.pacnew"), NEW).unwrap();
    root
}

fn merge(copy: &str, resolution: Resolution, current: Option<&str>) -> MergeResolve {
    merge_of(copy, NEW, resolution, current)
}

/// The resolve of `copy` as the user saw it holding `copied`.
fn merge_of(
    copy: &str,
    copied: &str,
    resolution: Resolution,
    current: Option<&str>,
) -> MergeResolve {
    MergeResolve {
        copy: MergeCopy::new(copy).unwrap(),
        choice: MergeChoice {
            resolution,
            copy: Fingerprint::of(copied.as_bytes()),
            current: current.map(|content| Fingerprint::of(content.as_bytes())),
        },
    }
}

fn pacnew(resolution: Resolution) -> MergeResolve {
    merge("/etc/pacman.conf.pacnew", resolution, Some(CURRENT))
}

fn no_check(command: &Command, _text: &ConfigText) -> Outcome {
    panic!("{command} ran for a file that needs no check")
}

fn unchecked(root: &Path, merge: &MergeResolve) -> Result<Resolved, MergeError> {
    resolve(root, merge, no_check)
}

/// Runs the step below `root` and returns how it ended and what it printed.
fn executed(root: &Path, merge: &MergeResolve) -> (Outcome, String) {
    let mut events = Vec::new();
    let outcome = execute(
        root,
        StepId::new(1),
        merge,
        &mut events,
        |command, text, _| no_check(command, text),
    );

    (outcome, printed(&events))
}

fn read(root: &TempDir, file: &str) -> Option<String> {
    fs::read_to_string(root.path().join(file)).ok()
}

#[test]
fn a_file_is_a_backup_only_when_a_package_lists_it_so() {
    let root = system();

    assert!(is_backup(root.path(), &path("/etc/pacman.conf")).unwrap());
    assert!(is_backup(root.path(), &path("/etc/sudoers")).unwrap());
    assert!(!is_backup(root.path(), &path("/etc/pacman.d/mirrorlist")).unwrap());
}

#[test]
fn keeping_the_current_file_removes_the_copy_only() {
    let root = system();

    assert_eq!(
        unchecked(root.path(), &pacnew(Resolution::KeepCurrent)).unwrap(),
        Resolved::Removed
    );
    assert_eq!(read(&root, "etc/pacman.conf").as_deref(), Some(CURRENT));
    assert_eq!(read(&root, "etc/pacman.conf.pacnew"), None);
}

#[test]
fn taking_the_copy_keeps_the_mode_and_backs_the_old_file_up() {
    let root = system();
    let mode = |file: &str| {
        fs::metadata(root.path().join(file))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777
    };

    assert_eq!(
        unchecked(root.path(), &pacnew(Resolution::TakeCopy)).unwrap(),
        Resolved::Written
    );
    assert_eq!(read(&root, "etc/pacman.conf").as_deref(), Some(NEW));
    assert_eq!(read(&root, "etc/pacman.conf.pacnew"), None);
    assert_eq!(
        read(&root, &format!("{BACKUPS}/etc/pacman.conf")).as_deref(),
        Some(CURRENT)
    );
    assert_eq!(mode("etc/pacman.conf"), 0o640);
}

#[test]
fn a_merged_text_takes_the_files_place() {
    let root = system();
    let merged = "Color\nParallelDownloads = 10\n";
    let resolved = unchecked(
        root.path(),
        &pacnew(Resolution::Merged { text: text(merged) }),
    );

    assert_eq!(resolved.unwrap(), Resolved::Written);
    assert_eq!(read(&root, "etc/pacman.conf").as_deref(), Some(merged));
    assert_eq!(read(&root, "etc/pacman.conf.pacnew"), None);
}

#[test]
fn a_copy_without_its_file_moves_into_its_place() {
    let root = system();
    fs::remove_file(root.path().join("etc/pacman.conf")).unwrap();

    let resolved = unchecked(
        root.path(),
        &merge("/etc/pacman.conf.pacnew", Resolution::TakeCopy, None),
    );

    assert_eq!(resolved.unwrap(), Resolved::Moved);
    assert_eq!(read(&root, "etc/pacman.conf").as_deref(), Some(NEW));
}

#[test]
fn a_file_that_changed_since_the_comparison_stays() {
    let root = system();
    fs::write(root.path().join("etc/pacman.conf"), "edited meanwhile\n").unwrap();

    let resolved = unchecked(root.path(), &pacnew(Resolution::TakeCopy));

    assert!(matches!(
        resolved,
        Err(MergeError::Write(WriteError::Changed))
    ));
    assert_eq!(read(&root, "etc/pacman.conf.pacnew").as_deref(), Some(NEW));
}

#[test]
fn a_copy_of_a_file_no_package_backs_up_is_refused() {
    let root = system();
    fs::create_dir_all(root.path().join("etc/pacman.d")).unwrap();
    fs::write(root.path().join("etc/pacman.d/mirrorlist.pacnew"), NEW).unwrap();

    let resolved = unchecked(
        root.path(),
        &merge(
            "/etc/pacman.d/mirrorlist.pacnew",
            Resolution::KeepCurrent,
            None,
        ),
    );

    assert!(matches!(resolved, Err(MergeError::NoBackup)));
    assert!(root.path().join("etc/pacman.d/mirrorlist.pacnew").exists());
}

#[test]
fn a_copy_that_is_a_symlink_is_refused() {
    let root = system();
    fs::remove_file(root.path().join("etc/pacman.conf.pacnew")).unwrap();
    fs::write(root.path().join("etc/elsewhere"), NEW).unwrap();
    symlink("elsewhere", root.path().join("etc/pacman.conf.pacnew")).unwrap();

    let resolved = unchecked(root.path(), &pacnew(Resolution::KeepCurrent));

    assert!(matches!(
        resolved,
        Err(MergeError::Write(WriteError::Refused(_)))
    ));
    assert_eq!(read(&root, "etc/elsewhere").as_deref(), Some(NEW));
}

#[test]
fn the_step_fails_and_says_why_when_a_file_changed() {
    let root = system();
    fs::write(root.path().join("etc/pacman.conf.pacnew"), "other\n").unwrap();

    let (outcome, printed) = executed(root.path(), &pacnew(Resolution::KeepCurrent));

    assert_failed(outcome);
    assert!(
        printed.contains("changed since they were compared"),
        "{printed}"
    );
}

#[test]
fn the_step_names_the_backup_after_a_write() {
    let root = system();
    let (outcome, printed) = executed(root.path(), &pacnew(Resolution::TakeCopy));

    assert_eq!(outcome, Outcome::Succeeded);
    assert!(
        printed.contains(&format!("/{BACKUPS}/etc/pacman.conf")),
        "{printed}"
    );
}

#[test]
fn fetch_may_hand_out_a_copy_and_the_file_beside_it() {
    let root = system();

    assert!(is_fetchable(root.path(), &path("/etc/pacman.conf.pacnew")));
    assert!(is_fetchable(root.path(), &path("/etc/pacman.conf")));
    assert!(!is_fetchable(root.path(), &path("/etc/sudoers")));
    assert!(!is_fetchable(root.path(), &path("/etc/fstab.pacnew")));
}

#[test]
fn a_snapshot_needs_snapper_and_its_root_configuration() {
    let root = tempfile::tempdir().unwrap();
    let place = |file: &str| {
        let file = root.path().join(file);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, "").unwrap();
    };

    assert_eq!(snapshot(root.path()), None);
    place("usr/bin/snapper");
    assert_eq!(snapshot(root.path()), None);
    place("etc/snapper/configs/root");

    assert_eq!(
        snapshot(root.path()).unwrap().to_string(),
        "snapper --no-dbus -c root create --cleanup-algorithm number --description 'Garuda Toolbox: before merging configuration files'"
    );
}
