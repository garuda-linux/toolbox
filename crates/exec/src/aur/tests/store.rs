use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::path::Path;

use rustix::fs::FlockOperation;
use rustix::io::Errno;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{AUR_STORE, Caller, InstallReason};

use super::{OCTOPI, octopi, passwd_with_caller, staged, system, uid};
use crate::aur::{StageError, empty_store, install};

/// A crash between the stage and the cleanup leaves a run folder behind.
#[test]
fn leftovers_of_another_run_are_gone_after_a_stage() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"").unwrap();
    let old = root.path().join(AUR_STORE).join("9f9f");
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("old-1-1-any.pkg.tar.zst"), b"").unwrap();

    staged(root.path(), &["octopi-git"]).unwrap();

    let left: Vec<_> = fs::read_dir(root.path().join(AUR_STORE))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();

    assert_eq!(left, ["1a2b"]);
}

#[test]
fn a_missing_or_empty_store_is_nothing_to_empty() {
    let root = tempfile::tempdir().unwrap();
    empty_store(root.path()).unwrap();

    fs::create_dir_all(root.path().join(AUR_STORE)).unwrap();
    empty_store(root.path()).unwrap();

    assert!(root.path().join(AUR_STORE).exists());
}

/// Anything but regular files in a run folder is not the helper's, so it stays
/// for a person to look at.
#[test]
fn a_folder_inside_a_run_folder_fails() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join(AUR_STORE);
    fs::create_dir_all(store.join("1a2b/nested")).unwrap();

    assert!(matches!(
        empty_store(root.path()),
        Err(StageError::NotRegular(name)) if name == "nested"
    ));
    assert!(store.join("1a2b/nested").exists());
}

#[test]
fn a_file_directly_in_the_store_fails() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join(AUR_STORE);
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("stray"), b"").unwrap();

    assert!(matches!(
        empty_store(root.path()),
        Err(StageError::NoRunFolder(name)) if name == "stray"
    ));
    assert!(store.join("stray").exists());
}

#[test]
fn a_linked_run_folder_fails_and_its_target_stays() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join(AUR_STORE);
    let elsewhere = root.path().join("elsewhere");
    fs::create_dir_all(&store).unwrap();
    fs::create_dir_all(&elsewhere).unwrap();
    fs::write(elsewhere.join("kept"), b"").unwrap();
    symlink(&elsewhere, store.join("1a2b")).unwrap();

    assert!(matches!(
        empty_store(root.path()),
        Err(StageError::NoRunFolder(name)) if name == "1a2b"
    ));
    assert!(elsewhere.join("kept").exists());
}

#[test]
fn a_link_inside_a_run_folder_fails_and_its_target_stays() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join(AUR_STORE);
    let kept = root.path().join("kept");
    fs::create_dir_all(store.join("1a2b")).unwrap();
    fs::write(&kept, b"").unwrap();
    symlink(&kept, store.join("1a2b/link")).unwrap();

    assert!(matches!(
        empty_store(root.path()),
        Err(StageError::NotRegular(name)) if name == "link"
    ));
    assert!(kept.exists());
}

fn try_lock(store: &Path) -> Result<File, Errno> {
    let folder = File::open(store).unwrap();
    rustix::fs::flock(&folder, FlockOperation::NonBlockingLockExclusive)?;
    Ok(folder)
}

#[test]
fn a_store_another_helper_holds_installs_nothing() {
    let (root, folder) = system();
    passwd_with_caller(root.path());
    fs::write(folder.join(OCTOPI), b"").unwrap();
    let other = root.path().join(AUR_STORE).join("9f9f");
    fs::create_dir_all(&other).unwrap();
    fs::write(other.join("old-1-1-any.pkg.tar.zst"), b"").unwrap();
    let _held = try_lock(&root.path().join(AUR_STORE)).unwrap();

    let outcome = install(
        root.path(),
        Caller::new(uid()),
        octopi(InstallReason::Explicit),
        |_command| panic!("pacman ran on a store another helper holds"),
    );

    assert!(matches!(outcome, Err(StageError::Busy)));
    assert!(other.join("old-1-1-any.pkg.tar.zst").exists());
}

#[test]
fn the_store_is_held_while_pacman_runs_and_free_after() {
    let (root, folder) = system();
    passwd_with_caller(root.path());
    fs::write(folder.join(OCTOPI), b"").unwrap();
    let store = root.path().join(AUR_STORE);

    install(
        root.path(),
        Caller::new(uid()),
        octopi(InstallReason::Explicit),
        |_command| {
            assert_eq!(try_lock(&store).unwrap_err(), Errno::WOULDBLOCK);
            Outcome::Succeeded
        },
    )
    .unwrap();

    try_lock(&store).unwrap();
}
