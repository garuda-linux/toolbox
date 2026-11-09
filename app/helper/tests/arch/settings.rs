//! The System Settings page's root actions in the Arch test image. The
//! image has no systemd, NetworkManager or hblock, so only what works
//! without them runs here: the login shell, the DNS drop-in and the
//! garuda-update switches. hblock and the Wi-Fi backend need a booted
//! system and are covered by the argv and executor tests.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, RootAction, ShellPath, UserName};

use super::common::{TestResult, helper, request};
use super::{apply, in_test_image, printed};

const DNS_FILE: &str = "/etc/NetworkManager/conf.d/10-garuda-assistant-dns.conf";
const GARUDA_UPDATE_FOLDER: &str = "/etc/garuda/garuda-update";
const GARUDA_UPDATE_FILE: &str = "/etc/garuda/garuda-update/config";

fn login_shell(user: &str) -> Result<String, Box<dyn Error>> {
    let entry = Command::new("getent").args(["passwd", user]).output()?;
    let entry = String::from_utf8_lossy(&entry.stdout);

    Ok(entry
        .trim_end()
        .rsplit(':')
        .next()
        .unwrap_or_default()
        .to_owned())
}

fn set_shell(shell: &str) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::SetShell {
        user: UserName::new("tester")?,
        shell: ShellPath::new(shell)?,
    })
}

fn listed(shell: &str) -> Result<bool, Box<dyn Error>> {
    Ok(fs::read_to_string("/etc/shells")?
        .lines()
        .any(|line| line.trim() == shell))
}

const fn failed(outcomes: &[Outcome]) -> bool {
    matches!(
        outcomes,
        [Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }]
    )
}

#[test]
fn a_listed_shell_becomes_the_login_shell() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    assert!(listed("/usr/bin/sh")? && listed("/usr/bin/bash")?);

    assert_eq!(
        apply(vec![set_shell("/usr/bin/sh")?])?,
        [Outcome::Succeeded]
    );
    assert_eq!(login_shell("tester")?, "/usr/bin/sh");

    assert_eq!(
        apply(vec![set_shell("/usr/bin/bash")?])?,
        [Outcome::Succeeded]
    );
    assert_eq!(login_shell("tester")?, "/usr/bin/bash");
    Ok(())
}

/// chsh as root would take `/usr/bin/false`. The helper must not.
#[test]
fn a_shell_missing_from_etc_shells_is_refused() -> TestResult {
    if !in_test_image() || listed("/usr/bin/false")? {
        return Ok(());
    }

    let before = login_shell("tester")?;
    let output = helper(
        &["apply"],
        &request(Mode::Run, vec![set_shell("/usr/bin/false")?])?,
    )?;

    assert!(!output.status.success());
    assert!(
        printed(&output)?.contains("is not listed in /etc/shells"),
        "{}",
        printed(&output)?
    );
    assert_eq!(login_shell("tester")?, before);
    Ok(())
}

/// The image has no NetworkManager, so `nmcli general reload` fails: the
/// file is written or removed, and the step reports the failed reload.
#[test]
fn the_dns_drop_in_is_written_and_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fs::create_dir_all("/etc/NetworkManager/conf.d")?;
    let reload_fails = !Path::new("/usr/bin/nmcli").exists();

    let written = apply(vec![RootAction::SetDns {
        provider: DnsProvider::Cloudflare,
    }])?;

    assert_eq!(
        fs::read_to_string(DNS_FILE)?,
        "[global-dns-domain-*]\nservers=1.1.1.1\n"
    );
    assert!(!reload_fails || failed(&written), "{written:?}");

    let removed = apply(vec![RootAction::SetDns {
        provider: DnsProvider::Default,
    }])?;

    assert!(!Path::new(DNS_FILE).exists());
    assert!(!reload_fails || failed(&removed), "{removed:?}");
    Ok(())
}

#[test]
fn garuda_update_switches_create_and_edit_the_config() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    if Path::new(GARUDA_UPDATE_FOLDER).exists() {
        fs::remove_dir_all(GARUDA_UPDATE_FOLDER)?;
    }

    let switch = |flag, enabled| RootAction::SetGarudaUpdateFlag { flag, enabled };

    assert_eq!(
        apply(vec![switch(GarudaUpdateFlag::UpdateAur, true)])?,
        [Outcome::Succeeded]
    );
    assert_eq!(fs::read_to_string(GARUDA_UPDATE_FILE)?, "UPDATE_AUR=1\n");

    fs::write(
        GARUDA_UPDATE_FILE,
        "# shipped\n# SKIP_MIRRORLIST=1\nUPDATE_AUR=1\nMINE=\"x\"\n",
    )?;

    assert_eq!(
        apply(vec![
            switch(GarudaUpdateFlag::SkipMirrorlist, true),
            switch(GarudaUpdateFlag::UpdateAur, false),
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );
    assert_eq!(
        fs::read_to_string(GARUDA_UPDATE_FILE)?,
        "# shipped\nSKIP_MIRRORLIST=1\n# UPDATE_AUR=1\nMINE=\"x\"\n"
    );

    assert_eq!(
        apply(vec![switch(GarudaUpdateFlag::UpdateAur, false)])?,
        [Outcome::Skipped],
        "already off"
    );
    Ok(())
}
