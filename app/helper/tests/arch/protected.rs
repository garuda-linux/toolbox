//! `chaotic-mirrorlist` of the test repository is protected by its name
//! and pulled in by `toolbox-test-on-protected` alone, so `-s` would take
//! it along with that package.

use std::process::Command;

use toolbox_plan::RootAction;
use toolbox_plan::protocol::Outcome;

use super::{apply, in_test_image, installed, packages, reset};
use crate::common::TestResult;

#[test]
fn a_removal_that_takes_a_protected_dependency_along_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    Command::new("pacman")
        .args(["-S", "--noconfirm", "toolbox-test-on-protected"])
        .status()?;
    assert!(installed("chaotic-mirrorlist"));

    let removed = apply(vec![RootAction::RemovePackages {
        packages: packages(&["toolbox-test-on-protected"])?,
    }])?;

    assert!(
        matches!(
            removed.as_slice(),
            [Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }]
        ),
        "{removed:?}"
    );
    assert!(installed("toolbox-test-on-protected"));
    assert!(installed("chaotic-mirrorlist"));
    Ok(())
}

#[test]
fn an_orphan_removal_keeps_the_protected_dependency_it_frees() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    Command::new("pacman")
        .args(["-S", "--asdeps", "--noconfirm", "toolbox-test-on-protected"])
        .status()?;
    assert!(installed("chaotic-mirrorlist"));

    let removed = apply(vec![RootAction::RemoveOrphans])?;

    assert_eq!(removed, [Outcome::Succeeded]);
    assert!(!installed("toolbox-test-on-protected"));
    assert!(installed("chaotic-mirrorlist"), "kept, as protected");
    Ok(())
}
