//! The Network page's root actions in the Arch test image. A container
//! may not load kernel modules and has no systemd, so what runs here is
//! the blacklist file on the real `/etc`, and a load that must fail on
//! the module alone while everything around it is missing.

use std::fs;
use std::path::Path;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{KernelModule, RootAction};

use super::common::{TestResult, helper, request};
use super::{apply, in_test_image, printed};

const OWN_FILE: &str = "/etc/modprobe.d/garuda-toolbox-blacklist.conf";
const OTHER_FILE: &str = "/etc/modprobe.d/toolbox-test-other.conf";
const PACKAGE_FILE: &str = "/usr/lib/modprobe.d/toolbox-test-package.conf";

fn blacklist(module: &str, blacklisted: bool) -> Result<RootAction, Box<dyn std::error::Error>> {
    Ok(RootAction::SetModuleBlacklisted {
        module: KernelModule::new(module)?,
        blacklisted,
    })
}

/// Only the app's own file changes, never a file of a package or of the
/// administrator.
#[test]
fn blacklisting_writes_the_apps_own_file_and_no_other() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fs::create_dir_all("/etc/modprobe.d")?;
    fs::create_dir_all("/usr/lib/modprobe.d")?;
    fs::write(OTHER_FILE, "blacklist toolbox_test_a\n")?;
    fs::write(PACKAGE_FILE, "blacklist toolbox_test_a\n")?;

    if Path::new(OWN_FILE).exists() {
        fs::remove_file(OWN_FILE)?;
    }

    assert_eq!(
        apply(vec![
            blacklist("toolbox_test_a", true)?,
            blacklist("toolbox-test-b", true)?,
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );
    assert_eq!(
        fs::read_to_string(OWN_FILE)?,
        "# Written by Garuda Toolbox (Network, Drivers). Edit it there.\n\
         blacklist toolbox_test_a\nblacklist toolbox_test_b\n"
    );

    assert_eq!(
        apply(vec![blacklist("toolbox_test_b", true)?])?,
        [Outcome::Skipped],
        "already listed"
    );
    assert_eq!(
        apply(vec![
            blacklist("toolbox_test_a", false)?,
            blacklist("toolbox_test_b", false)?,
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );
    assert!(!Path::new(OWN_FILE).exists(), "the empty file stays behind");
    assert_eq!(
        fs::read_to_string(OTHER_FILE)?,
        "blacklist toolbox_test_a\n"
    );
    assert_eq!(
        fs::read_to_string(PACKAGE_FILE)?,
        "blacklist toolbox_test_a\n"
    );

    fs::remove_file(OTHER_FILE)?;
    fs::remove_file(PACKAGE_FILE)?;
    Ok(())
}

/// No NetworkManager in the image: the strict load fails at its first
/// part and never reaches `modprobe`.
#[test]
fn loading_a_module_fails_without_network_manager() -> TestResult {
    if !in_test_image() || !Path::new("/usr/bin/modprobe").exists() {
        return Ok(());
    }

    let output = helper(
        &["apply"],
        &request(
            Mode::Run,
            vec![RootAction::LoadModule {
                module: KernelModule::new("toolbox_test_no_such_module")?,
            }],
        )?,
    )?;

    let text = printed(&output)?;

    assert!(!output.status.success());
    assert!(
        !text.contains("toolbox_test_no_such_module not found"),
        "{text}"
    );
    Ok(())
}

#[test]
fn unloading_a_module_that_is_not_there_fails() -> TestResult {
    if !in_test_image() || !Path::new("/usr/bin/modprobe").exists() {
        return Ok(());
    }

    let outcomes = apply(vec![RootAction::UnloadModule {
        module: KernelModule::new("toolbox_test_no_such_module")?,
    }])?;

    assert!(
        matches!(
            outcomes.as_slice(),
            [Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }]
        ),
        "{outcomes:?}"
    );
    Ok(())
}
