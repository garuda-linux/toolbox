//! The removal of make dependencies after an AUR build, against pacman in
//! the Arch test image. A dependency nothing needs goes, one something
//! requires stays, an explicit package is never touched and a protected
//! name is refused before anything runs.

use std::error::Error;
use std::process::Command;

use toolbox_plan::RootAction;
use toolbox_plan::protocol::{Event, Mode, Outcome};

use super::{apply, in_test_image, installed, packages, reset};
use crate::common::{TestResult, events, helper, request};

fn removal(names: &[&str]) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::RemoveMakeDependencies {
        packages: packages(names)?,
    })
}

/// Installs `args` with pacman on top of a clean test image.
fn installing(args: &[&str]) -> TestResult {
    reset()?;
    Command::new("pacman").args(args).status()?;
    Ok(())
}

#[test]
fn a_dependency_nothing_needs_is_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    installing(&["-S", "--asdeps", "--noconfirm", "toolbox-test-dep"])?;
    assert!(installed("toolbox-test-dep"));

    let outcomes = apply(vec![removal(&["toolbox-test-dep"])?])?;

    assert_eq!(outcomes, [Outcome::Succeeded]);
    assert!(!installed("toolbox-test-dep"));
    Ok(())
}

#[test]
fn a_dependency_something_requires_stays_and_the_step_succeeds() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    installing(&["-S", "--noconfirm", "toolbox-test-a"])?;
    assert!(installed("toolbox-test-dep"));

    let outcomes = apply(vec![removal(&["toolbox-test-dep"])?])?;

    assert_eq!(outcomes, [Outcome::Skipped]);
    assert!(installed("toolbox-test-a") && installed("toolbox-test-dep"));
    Ok(())
}

#[test]
fn an_explicitly_installed_package_is_never_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    installing(&["-S", "--noconfirm", "toolbox-test-dep"])?;
    assert!(installed("toolbox-test-dep"));

    let outcomes = apply(vec![removal(&["toolbox-test-dep"])?])?;

    assert_eq!(outcomes, [Outcome::Skipped]);
    assert!(installed("toolbox-test-dep"));
    Ok(())
}

#[test]
fn a_protected_make_dependency_is_refused_before_anything_runs() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    installing(&["-S", "--noconfirm", "toolbox-test-on-protected"])?;
    assert!(installed("chaotic-mirrorlist"));

    let input = request(Mode::Run, vec![removal(&["chaotic-mirrorlist"])?])?;
    let output = helper(&["apply"], &input)?;

    assert!(
        events(&output)?
            .iter()
            .any(|event| matches!(event, Event::Rejected { reason: _ })),
        "{}",
        output.stdout
    );
    assert!(installed("chaotic-mirrorlist"));
    assert!(installed("toolbox-test-on-protected"));
    Ok(())
}
