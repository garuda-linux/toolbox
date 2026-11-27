//! The refusals of `InstallBuilt` against real pacman: each setup that the
//! helper must not install from fails its step and installs nothing, and the
//! root store is left empty or untouched.

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, chown, symlink};
use std::path::Path;

use rustix::fs::{FlockOperation, flock};
use toolbox_plan::protocol::{Answer, Mode, Outcome};
use toolbox_plan::{AUR_STORE, BuildRun, InstallReason, RootAction};

use super::aur::{
    HOME, PACKAGE, UNKNOWN_UID, account, build, built_file, install, piped, run_folder,
    store_is_empty,
};
use super::{in_test_image, installed, packages, printed, reset};
use crate::common::{TestResult, outcomes, request};

const fn refused(results: &[Outcome]) -> bool {
    matches!(
        results,
        [Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }]
    )
}

/// Asks for `run` as `uid`, which must be refused and install nothing. The
/// printed text comes back for the reason check.
fn refused_for(uid: u32, run: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (results, text) = install(uid, run, InstallReason::Explicit)?;

    assert!(refused(&results), "{text}");
    assert!(!installed(PACKAGE));
    Ok(text)
}

/// A clean start: no package installed, and `run` built by `ada`.
fn fresh(run: &str) -> TestResult {
    reset()?;
    build(run, "")
}

#[test]
fn a_pipe_on_stdin_names_no_caller_and_installs_nothing() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("piped")?;

    let text = request(
        Mode::Run,
        vec![RootAction::InstallBuilt {
            run: BuildRun::new("piped")?,
            packages: packages(&[PACKAGE])?,
            reason: InstallReason::Explicit,
        }],
    )?;

    let output = piped(&text, |step| Some(Answer::Proceed { step }))?;
    let printed = printed(&output)?;

    assert!(refused(&outcomes(&output)?), "{printed}");
    assert!(printed.contains("does not know who asked"), "{printed}");
    assert!(!installed(PACKAGE));
    Ok(())
}

#[test]
fn a_run_folder_writable_by_its_group_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("group")?;
    fs::set_permissions(run_folder("group"), fs::Permissions::from_mode(0o770))?;

    let text = refused_for(account()?.0, "group")?;

    assert!(text.contains("may be written by other users"), "{text}");
    Ok(())
}

#[test]
fn a_cache_that_is_a_link_is_refused_and_nothing_is_moved() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("link")?;

    let cache = Path::new(HOME).join(".cache");
    let moved = Path::new(HOME).join(".cache-moved");
    fs::rename(&cache, &moved)?;
    symlink(&moved, &cache)?;

    let result = refused_for(account()?.0, "link");

    fs::remove_file(&cache)?;
    fs::rename(&moved, &cache)?;
    result.map(|_text| ())
}

#[test]
fn an_uid_without_a_passwd_entry_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("nobody")?;

    let text = refused_for(UNKNOWN_UID, "nobody")?;

    assert!(text.contains("has no account in /etc/passwd"), "{text}");
    Ok(())
}

#[test]
fn a_cache_owned_by_another_uid_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("owner")?;
    let (uid, gid) = account()?;
    let cache = Path::new(HOME).join(".cache");

    chown(&cache, Some(0), Some(0))?;
    let result = refused_for(uid, "owner");
    chown(&cache, Some(uid), Some(gid))?;

    let text = result?;

    assert!(
        text.contains("does not belong to the user who asked"),
        "{text}"
    );
    Ok(())
}

#[test]
fn a_package_file_that_is_a_link_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("linked")?;

    let file = built_file("linked")?;
    fs::remove_file(&file)?;
    symlink("/etc/os-release", &file)?;

    let text = refused_for(account()?.0, "linked")?;

    assert!(text.contains(PACKAGE), "{text}");
    Ok(())
}

#[test]
fn a_package_file_owned_by_another_uid_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("foreign")?;
    chown(built_file("foreign")?, Some(0), Some(0))?;

    let text = refused_for(account()?.0, "foreign")?;

    assert!(
        text.contains("does not belong to the user who asked"),
        "{text}"
    );
    Ok(())
}

/// pacman refuses the copy, and the store is emptied all the same.
#[test]
fn a_corrupt_package_fails_pacman_and_leaves_the_store_empty() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("corrupt")?;
    fs::write(built_file("corrupt")?, b"not a package")?;

    refused_for(account()?.0, "corrupt")?;

    assert!(store_is_empty()?);
    Ok(())
}

#[test]
fn a_held_store_lock_makes_the_install_busy_and_touches_nothing() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh("busy")?;

    let store = Path::new("/").join(AUR_STORE);
    let other = store.join("other-run");
    fs::create_dir_all(&other)?;
    fs::write(other.join("marker"), "kept")?;

    let held = fs::File::open(&store)?;
    flock(&held, FlockOperation::NonBlockingLockExclusive)?;

    let result = install(account()?.0, "busy", InstallReason::Explicit);
    drop(held);

    let marker_kept = other.join("marker").exists();
    let package_kept = built_file("busy").is_ok();
    fs::remove_dir_all(&other)?;

    let (results, text) = result?;

    assert!(refused(&results), "{text}");
    assert!(
        text.contains("another install of built packages is using"),
        "{text}"
    );
    assert!(marker_kept && package_kept && !installed(PACKAGE));
    Ok(())
}
