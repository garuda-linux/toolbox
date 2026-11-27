use std::fs;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use rustix::fs::Mode;
use toolbox_plan::{AUR_BUILT, BuildRun, PARU_CLONE, PackageName};

use super::{BuildError, prepare};
use crate::aur::built_folder;
use crate::aur::route::RouteProblem;
use crate::test_support::{ada_home, mode};

const OCTOPI: &str = "octopi-git";

fn run() -> BuildRun {
    BuildRun::new("1a2b").unwrap()
}

fn clone_of(home: &Path) -> PathBuf {
    home.join(PARU_CLONE).join(OCTOPI)
}

fn prepare_now(home: &Path) -> Result<toolbox_pty::PtyCommand, BuildError> {
    prepare(
        home,
        Path::new("/usr/bin/makepkg"),
        &PackageName::new(OCTOPI).unwrap(),
        &run(),
        SystemTime::now(),
    )
}

#[track_caller]
fn refused_naming_the_clone(home: &Path) {
    let error = prepare_now(home).unwrap_err();

    assert!(
        error.to_string().contains("~/.cache/paru/clone/octopi-git"),
        "{error}"
    );
    assert!(!home.join(AUR_BUILT).exists());
}

#[test]
fn a_missing_clone_is_refused_by_name() {
    let (_root, home) = ada_home();

    refused_naming_the_clone(&home);
}

#[test]
fn a_linked_clone_is_refused_and_not_followed() {
    let (root, home) = ada_home();
    let target = root.path().join("target");
    fs::create_dir_all(home.join(PARU_CLONE)).unwrap();
    fs::create_dir_all(&target).unwrap();
    symlink(&target, clone_of(&home)).unwrap();

    refused_naming_the_clone(&home);
}

#[test]
fn a_group_writable_cache_is_refused_before_anything_is_made() {
    let (_root, home) = ada_home();
    fs::create_dir_all(clone_of(&home)).unwrap();
    fs::set_permissions(home.join(".cache"), fs::Permissions::from_mode(0o775)).unwrap();

    let error = prepare_now(&home).unwrap_err();

    assert!(
        matches!(&error, BuildError::Route(RouteProblem::Shared(name)) if name == ".cache"),
        "{error}"
    );
    assert!(!home.join(".cache/garuda-toolbox").exists());
}

#[test]
fn the_helper_accepts_the_run_folder_it_gets() {
    let (root, home) = ada_home();
    fs::create_dir_all(clone_of(&home)).unwrap();
    prepare_now(&home).unwrap();

    let uid = rustix::process::geteuid().as_raw();
    built_folder(root.path(), uid, Path::new("home/ada"), &run()).unwrap();
}

#[test]
fn old_run_folders_go_and_fresh_ones_and_links_stay() {
    let (root, home) = ada_home();
    fs::create_dir_all(clone_of(&home)).unwrap();
    let built = home.join(AUR_BUILT);
    let target = root.path().join("target");
    fs::create_dir_all(&target).unwrap();
    fs::create_dir_all(built.join("old")).unwrap();
    fs::create_dir_all(built.join("fresh")).unwrap();
    fs::File::create(target.join("keep")).unwrap();
    symlink(&target, built.join("linked")).unwrap();

    let two_days_ago = SystemTime::now() - Duration::from_hours(48);
    fs::File::open(built.join("old"))
        .unwrap()
        .set_modified(two_days_ago)
        .unwrap();

    prepare_now(&home).unwrap();

    assert!(!built.join("old").exists());
    assert!(built.join("fresh").is_dir());
    assert!(built.join("1a2b").is_dir());
    assert!(built.join("linked").is_symlink());
    assert!(target.join("keep").exists());
}

#[test]
fn the_run_folder_is_0700_under_umask_002() {
    assert_run_folder_mode(0o002);
}

#[test]
fn the_run_folder_is_0700_under_umask_077() {
    assert_run_folder_mode(0o077);
}

/// nextest runs each test in a process of its own, so the umask set here
/// reaches no other test.
fn assert_run_folder_mode(umask: u32) {
    let (_root, home) = ada_home();
    fs::create_dir_all(clone_of(&home)).unwrap();

    let previous = rustix::process::umask(Mode::from_raw_mode(umask));
    let made = prepare_now(&home);
    rustix::process::umask(previous);
    made.unwrap();

    let folder = home.join(AUR_BUILT).join("1a2b");
    assert_eq!(mode(&folder), 0o700, "umask {umask:o}");
    assert_eq!(
        fs::metadata(&folder).unwrap().uid(),
        rustix::process::geteuid().as_raw()
    );
}
