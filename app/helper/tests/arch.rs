//! The helper in run mode, as root, against pacman in the Arch test image
//! (`ci/arch/Containerfile`). Every test changes the system, so they only
//! run there: `TOOLBOX_IT_ARCH=1`, root and `/etc/arch-release`. Elsewhere
//! they return at once. Run them one at a time (`scripts/arch-it.sh`).

#[path = "arch/boot.rs"]
mod boot;
mod common;
#[path = "arch/configs.rs"]
mod configs;
#[path = "arch/diagnostics.rs"]
mod diagnostics;
#[path = "arch/groups.rs"]
mod groups;
#[path = "arch/homes.rs"]
mod homes;
#[path = "arch/locales.rs"]
mod locales;
#[path = "arch/network.rs"]
mod network;
mod oneshot;
#[path = "arch/passwords.rs"]
mod passwords;
#[path = "arch/repair.rs"]
mod repair;
mod repair_names;
#[path = "arch/settings.rs"]
mod settings;
#[path = "arch/users.rs"]
mod users;

use std::error::Error;
use std::io::Write as _;
use std::path::Path;
use std::process::Command;

use common::{Reply, Run, TestResult, events, helper, helper_answering, outcomes, request};
use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{
    GroupName, Mirrorlist, PackageName, RootAction, TargetList, Upgrader, UserName,
};

const TEST_PACKAGES: [&str; 4] = [
    "toolbox-test-a",
    "toolbox-test-dep",
    "toolbox-test-conflict",
    "toolbox-test-up",
];

/// Runs `actions` for real and returns every step's outcome.
/// When a step did not succeed, the helper's own messages and the programs'
/// output go to the test's stderr, which `cargo test` shows on failure.
fn apply(actions: Vec<RootAction>) -> Result<Vec<Outcome>, Box<dyn Error>> {
    apply_answering(actions, None)
}

/// [`apply`] that answers a question of the running program, as a person
/// in the Console does. A program that asks waits for it, else for ever.
fn apply_answering(
    actions: Vec<RootAction>,
    reply: Option<Reply<'_>>,
) -> Result<Vec<Outcome>, Box<dyn Error>> {
    let output = helper_answering(&["apply"], &request(Mode::Run, actions)?, reply)?;
    let outcomes = outcomes(&output)?;

    let all_fine = outcomes
        .iter()
        .all(|outcome| matches!(outcome, Outcome::Succeeded | Outcome::Skipped));

    if !all_fine || outcomes.is_empty() {
        writeln!(
            std::io::stderr(),
            "helper exit: {}\nhelper stderr:\n{}\nevents:\n{}\nprinted:\n{}",
            output.status,
            output.stderr,
            output.stdout,
            printed(&output)?
        )?;
    }

    Ok(outcomes)
}

/// Everything the steps printed, as the Console shows it.
fn printed(output: &Run) -> Result<String, Box<dyn Error>> {
    Ok(events(output)?
        .into_iter()
        .filter_map(|event| match event {
            Event::Output { step: _, data } => Some(data),
            Event::Accepted
            | Event::Rejected { reason: _ }
            | Event::Started { step: _ }
            | Event::Finished {
                step: _,
                outcome: _,
            }
            | Event::Done => None,
        })
        .collect())
}

fn in_test_image() -> bool {
    std::env::var("TOOLBOX_IT_ARCH").as_deref() == Ok("1")
        && rustix::process::geteuid().is_root()
        && Path::new("/etc/arch-release").exists()
}

/// A file's bytes, or `None` when it does not exist.
fn saved(path: &str) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn restore(path: &str, bytes: Option<&[u8]>) -> TestResult {
    match bytes {
        Some(bytes) => std::fs::write(path, bytes)?,
        None if Path::new(path).exists() => std::fs::remove_file(path)?,
        None => {}
    }

    Ok(())
}

fn installed(name: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", name])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn version(name: &str) -> String {
    Command::new("pacman")
        .args(["-Q", name])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default()
}

fn packages(names: &[&str]) -> Result<TargetList<PackageName>, Box<dyn Error>> {
    let names: Result<Vec<PackageName>, _> =
        names.iter().map(|name| PackageName::new(*name)).collect();

    Ok(TargetList::new(names?)?)
}

/// Leaves only `toolbox-test-up` installed, as the image starts.
fn reset() -> TestResult {
    let present: Vec<&str> = TEST_PACKAGES
        .into_iter()
        .filter(|name| *name != "toolbox-test-up" && installed(name))
        .collect();

    if !present.is_empty() {
        Command::new("pacman")
            .args(["-Rdd", "--noconfirm"])
            .args(&present)
            .status()?;
    }

    Ok(())
}

#[test]
fn installing_pulls_in_dependencies_and_removing_cleans_them_up() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;

    let installed_now = apply(vec![RootAction::InstallPackages {
        packages: packages(&["toolbox-test-a"])?,
    }])?;

    assert_eq!(installed_now, [Outcome::Succeeded]);
    assert!(installed("toolbox-test-a") && installed("toolbox-test-dep"));

    let removed = apply(vec![RootAction::RemovePackages {
        packages: packages(&["toolbox-test-a"])?,
    }])?;

    assert_eq!(removed, [Outcome::Succeeded]);
    assert!(!installed("toolbox-test-a"));
    assert!(
        !installed("toolbox-test-dep"),
        "-Rns removes the dependency nothing needs"
    );
    Ok(())
}

#[test]
fn orphans_are_found_and_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    Command::new("pacman")
        .args(["-S", "--asdeps", "--noconfirm", "toolbox-test-dep"])
        .status()?;
    assert!(installed("toolbox-test-dep"));

    // pacman lists the orphans and asks before it removes them.
    assert_eq!(
        apply_answering(
            vec![RootAction::RemoveOrphans],
            Some(Reply {
                asked: "[Y/n]",
                typed: "y\n",
            }),
        )?,
        [Outcome::Succeeded]
    );
    assert!(!installed("toolbox-test-dep"));

    assert_eq!(
        apply(vec![RootAction::RemoveOrphans])?,
        [Outcome::Skipped],
        "no orphans left"
    );
    Ok(())
}

#[test]
fn the_pacman_upgrade_takes_the_newer_version() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    Command::new("pacman")
        .args([
            "-U",
            "--noconfirm",
            "/srv/toolbox-test-older/toolbox-test-up-1.0-1-any.pkg.tar.zst",
        ])
        .status()?;
    assert_eq!(version("toolbox-test-up"), "toolbox-test-up 1.0-1");

    let upgraded = apply(vec![RootAction::UpgradeSystem {
        upgrader: Upgrader::Pacman,
    }])?;

    assert_eq!(upgraded, [Outcome::Succeeded]);
    assert_eq!(version("toolbox-test-up"), "toolbox-test-up 1.1-1");
    Ok(())
}

#[test]
fn a_conflict_answered_no_fails_the_step_and_stops_the_plan() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    apply(vec![RootAction::InstallPackages {
        packages: packages(&["toolbox-test-a"])?,
    }])?;

    let output = helper(
        &["apply"],
        &request(
            Mode::Run,
            vec![
                RootAction::InstallPackages {
                    packages: packages(&["toolbox-test-conflict"])?,
                },
                RootAction::CleanPackageCache,
            ],
        )?,
    )?;

    let text = printed(&output)?;

    assert!(!output.status.success());
    assert!(
        text.contains("are in conflict"),
        "pacman's question reaches the console:\n{text}"
    );
    assert!(installed("toolbox-test-a") && !installed("toolbox-test-conflict"));
    Ok(())
}

#[test]
fn the_database_lock_is_removed_when_pacman_does_not_run() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    std::fs::write("/var/lib/pacman/db.lck", "")?;

    assert_eq!(
        apply(vec![RootAction::RemoveDatabaseLock])?,
        [Outcome::Succeeded]
    );
    assert!(!Path::new("/var/lib/pacman/db.lck").exists());
    assert_eq!(
        apply(vec![RootAction::RemoveDatabaseLock])?,
        [Outcome::Skipped],
        "no lock left"
    );
    Ok(())
}

#[test]
fn the_package_cache_is_cleaned() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    assert_eq!(
        apply(vec![RootAction::CleanPackageCache])?,
        [Outcome::Succeeded]
    );
    Ok(())
}

#[test]
fn group_membership_changes_both_ways() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let user = UserName::new("tester")?;
    let group = GroupName::new("toolbox-test-group")?;
    let members = || -> Result<String, Box<dyn Error>> {
        Ok(String::from_utf8_lossy(
            &Command::new("getent")
                .args(["group", "toolbox-test-group"])
                .output()?
                .stdout,
        )
        .into_owned())
    };

    assert_eq!(
        apply(vec![RootAction::AddToGroup {
            user: user.clone(),
            group: group.clone()
        }])?,
        [Outcome::Succeeded]
    );
    assert!(members()?.trim_end().ends_with("tester"), "{}", members()?);

    assert_eq!(
        apply(vec![RootAction::RemoveFromGroup { user, group }])?,
        [Outcome::Succeeded]
    );
    assert!(!members()?.contains("tester"), "{}", members()?);
    Ok(())
}

#[test]
fn the_chaotic_mirrorlist_is_written_with_a_backup() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let target = Path::new("/etc/pacman.d/chaotic-mirrorlist");

    std::fs::write(target, "Server = https://old.example.org/$repo/$arch\n")?;

    let mirrorlist =
        Mirrorlist::new("# ranked\nServer = https://cdn-mirror.chaotic.cx/$repo/$arch\n")?;

    assert_eq!(
        apply(vec![RootAction::InstallMirrorlist { mirrorlist }])?,
        [Outcome::Succeeded]
    );
    assert!(std::fs::read_to_string(target)?.contains("cdn-mirror.chaotic.cx"));
    assert!(
        std::fs::read_to_string("/etc/pacman.d/chaotic-mirrorlist.bak")?
            .contains("old.example.org")
    );
    Ok(())
}

/// garuda-chroot is Garuda's own tool and not in the Arch image: the step
/// fails cleanly instead of hanging on a missing program.
#[test]
fn the_chroot_fails_cleanly_without_garuda_chroot() -> TestResult {
    if !in_test_image() || Path::new("/usr/bin/garuda-chroot").exists() {
        return Ok(());
    }

    let outcomes = apply(vec![RootAction::ChrootInstallation])?;

    assert!(
        matches!(outcomes.as_slice(), [Outcome::Failed { cause: _ }]),
        "the missing program must fail the step"
    );
    Ok(())
}
