use std::fs;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rustix::fs::Mode;
use toolbox_plan::{AUR_BUILT, AUR_CLONE, BuildRun, PackageName};

use super::{BuildError, prepare};
use crate::aur::built_folder;
use crate::aur::review::Unreviewed;
use crate::aur::route::RouteProblem;
use crate::test_support::{ada_home, git_program, mode, reviewed_clone};

const OCTOPI: &str = "octopi-git";

fn run() -> BuildRun {
    BuildRun::new("1a2b").unwrap()
}

fn clone_of(home: &Path) -> PathBuf {
    home.join(AUR_CLONE).join(OCTOPI)
}

fn prepare_now(home: &Path) -> Result<toolbox_pty::PtyCommand, BuildError> {
    let git = git_program();

    let octopi = PackageName::new(OCTOPI).unwrap();

    prepare(
        home,
        (Path::new("/usr/bin/makepkg"), git.as_path()),
        (&octopi, &run()),
        SystemTime::now(),
        &Arc::default(),
    )
}

#[track_caller]
fn refused_naming_the_clone(home: &Path) {
    let error = prepare_now(home).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("~/.cache/garuda-toolbox/aur-clone/octopi-git"),
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
    fs::create_dir_all(home.join(AUR_CLONE)).unwrap();
    fs::create_dir_all(&target).unwrap();
    symlink(&target, clone_of(&home)).unwrap();

    refused_naming_the_clone(&home);
}

#[test]
fn a_group_writable_cache_is_refused_before_anything_is_made() {
    let (_root, home) = ada_home();
    reviewed_clone(&home, OCTOPI);
    fs::set_permissions(home.join(".cache"), fs::Permissions::from_mode(0o775)).unwrap();

    let error = prepare_now(&home).unwrap_err();

    assert!(
        matches!(&error, BuildError::Route(RouteProblem::Shared(name)) if name == ".cache"),
        "{error}"
    );
    assert!(!home.join(AUR_BUILT).exists());
}

#[test]
fn the_helper_accepts_the_run_folder_it_gets() {
    let (root, home) = ada_home();
    reviewed_clone(&home, OCTOPI);
    prepare_now(&home).unwrap();

    let uid = rustix::process::geteuid().as_raw();
    built_folder(root.path(), uid, Path::new("home/ada"), &run()).unwrap();
}

#[test]
fn old_run_folders_go_and_fresh_ones_and_links_stay() {
    let (root, home) = ada_home();
    reviewed_clone(&home, OCTOPI);
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
    reviewed_clone(&home, OCTOPI);

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

fn git(folder: &Path, args: &[&str]) {
    let status = std::process::Command::new(git_program())
        .current_dir(folder)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args([
            "-c",
            "user.name=ada",
            "-c",
            "user.email=ada@example.invalid",
            "-c",
            "commit.gpgSign=false",
        ])
        .args(args)
        .status()
        .unwrap();

    assert!(status.success(), "git {args:?}");
}

#[test]
fn a_clone_that_moved_past_its_review_is_not_built() {
    let (_root, home) = ada_home();
    let clone = reviewed_clone(&home, OCTOPI);
    git(&clone, &["commit", "-q", "--allow-empty", "-m", "moved"]);

    let error = prepare_now(&home).unwrap_err();

    assert!(matches!(
        error,
        BuildError::Unreviewed {
            base: _,
            reason: Unreviewed::Changed,
        }
    ));
    assert!(
        error
            .to_string()
            .contains("the build files changed after the review")
    );
}

#[test]
fn a_clone_never_reviewed_is_not_built() {
    let (_root, home) = ada_home();
    let clone = reviewed_clone(&home, OCTOPI);
    git(
        &clone,
        &["update-ref", "-d", "refs/garuda-toolbox/reviewed"],
    );

    let error = prepare_now(&home).unwrap_err();

    assert!(matches!(
        error,
        BuildError::Unreviewed {
            base: _,
            reason: Unreviewed::NotReviewed,
        }
    ));
    assert!(error.to_string().contains("has no review yet"));
}

#[test]
fn a_tracked_file_changed_after_the_review_is_not_built() {
    let (_root, home) = ada_home();
    let clone = reviewed_clone(&home, OCTOPI);
    fs::write(clone.join("PKGBUILD"), "pkgname=octopi-git\n# edited\n").unwrap();

    let error = prepare_now(&home).unwrap_err();

    assert!(matches!(
        error,
        BuildError::Unreviewed {
            base: _,
            reason: Unreviewed::Changed,
        }
    ));
}

#[test]
fn untracked_files_do_not_stop_a_reviewed_build() {
    let (_root, home) = ada_home();
    let clone = reviewed_clone(&home, OCTOPI);
    fs::write(clone.join("octopi-git.tar.gz"), "source").unwrap();

    prepare_now(&home).unwrap();
}
