//! The image has neither Plymouth nor Garuda's `kernel-install-uki`, so
//! their steps are checked only where they are missing.

use std::fs;
use std::path::Path;

use toolbox_plan::protocol::Outcome;
use toolbox_plan::{KernelParameters, PlymouthTheme, RootAction};

use super::common::TestResult;
use super::{apply, in_test_image, restore, saved};

const CMDLINE: &str = "/etc/kernel/cmdline";
const BACKUP: &str = "/etc/kernel/cmdline.bak";
const FOLDER: &str = "/etc/kernel";
const SET_THEME: &str = "/usr/bin/plymouth-set-default-theme";
const UKI_TOOLS: [&str; 2] = [
    "/usr/lib/garuda/kernel-install-uki",
    "/usr/bin/kernel-install",
];

/// `/etc/kernel/cmdline` of the Garuda Lima image.
const IMAGE: &str = include_str!("../../../../crates/state/src/boot/fixtures/kernel-cmdline");

#[test]
fn the_kernel_cmdline_is_written_and_the_old_one_kept() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let made_folder = !Path::new(FOLDER).exists();
    let (cmdline, backup) = (saved(CMDLINE)?, saved(BACKUP)?);

    fs::create_dir_all(FOLDER)?;
    fs::write(CMDLINE, IMAGE)?;

    let parameters = KernelParameters::parse(IMAGE)
        .ok_or("the image's command line parses")?
        .without("quiet");

    let first = apply(vec![RootAction::SetKernelCmdline {
        parameters: parameters.clone(),
    }]);

    let written = fs::read_to_string(CMDLINE);
    let kept = fs::read_to_string(BACKUP);
    let again = apply(vec![RootAction::SetKernelCmdline { parameters }]);

    restore(CMDLINE, cmdline.as_deref())?;
    restore(BACKUP, backup.as_deref())?;

    if made_folder {
        fs::remove_dir(FOLDER)?;
    }

    assert_eq!(first?, [Outcome::Succeeded]);
    assert_eq!(
        written?,
        "root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6 rootflags=subvol=@ rw loglevel=3 console=tty0 console=ttyS0,115200\n"
    );
    assert_eq!(kept?, IMAGE);
    assert_eq!(again?, [Outcome::Skipped]);
    Ok(())
}

#[test]
fn without_plymouth_selecting_a_theme_fails() -> TestResult {
    if !in_test_image() || Path::new(SET_THEME).exists() {
        return Ok(());
    }

    let outcomes = apply(vec![RootAction::SelectPlymouthTheme {
        theme: PlymouthTheme::new("bgrt")?,
    }])?;

    assert!(
        matches!(outcomes.as_slice(), [Outcome::Failed { cause: _ }]),
        "{outcomes:?}"
    );
    Ok(())
}

#[test]
fn without_a_uki_tool_the_rebuild_fails() -> TestResult {
    if !in_test_image() || UKI_TOOLS.iter().any(|tool| Path::new(tool).exists()) {
        return Ok(());
    }

    let outcomes = apply(vec![RootAction::RebuildUkis { initramfs: false }])?;

    assert!(
        matches!(outcomes.as_slice(), [Outcome::Failed { cause: _ }]),
        "{outcomes:?}"
    );
    Ok(())
}
