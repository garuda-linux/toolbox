//! The AUR split path against real pacman and real makepkg. `ada`, the test
//! user `ci/arch/setup.sh` makes, builds a package in the folders the app
//! makes, and the helper installs what it built. The helper learns `ada`
//! from the peer of its stdin socket, as it learns the app's user.

use std::error::Error;
use std::os::unix::fs::{PermissionsExt as _, chown};
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

use toolbox_plan::protocol::{Answer, Mode, Outcome};
use toolbox_plan::{
    AUR_BUILT, AUR_CLONE, AUR_STORE, BuildRun, InstallReason, MAKEPKG_ARGS, RootAction, StepId,
    passwd_entries,
};

use super::{apply, in_test_image, installed, packages, printed, reset};
use crate::common::{Run, TestResult, drive, outcomes, request, run_helper_at};
use crate::person;

/// The package `ada` builds: no sources, one file, `arch=any`.
pub const PACKAGE: &str = "toolbox-test-aur";

/// An uid no account has, so the helper refuses it for its missing entry.
pub const UNKNOWN_UID: u32 = 4242;

const USER: &str = "ada";
pub const HOME: &str = "/home/ada";

/// `ada`'s uid and gid, from the image's passwd file.
pub fn account() -> Result<(u32, u32), Box<dyn Error>> {
    let passwd = fs::read_to_string("/etc/passwd")?;

    passwd_entries(&passwd)
        .find(|entry| entry.name == USER)
        .map(|entry| (entry.uid, entry.gid))
        .ok_or_else(|| "ada has no entry in /etc/passwd, ci/arch/setup.sh makes it".into())
}

pub fn run_folder(run: &str) -> PathBuf {
    Path::new(HOME).join(AUR_BUILT).join(run)
}

/// The one package file of `run`, as makepkg left it.
pub fn built_file(run: &str) -> Result<PathBuf, Box<dyn Error>> {
    fs::read_dir(run_folder(run))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.to_string_lossy().ends_with(".pkg.tar.zst"))
        .ok_or_else(|| "makepkg left no package file".into())
}

/// Removes `ada`'s cache, a link or a folder, so each build starts clean.
pub fn fresh_cache() -> TestResult {
    let cache = Path::new(HOME).join(".cache");

    match fs::symlink_metadata(&cache) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(&cache)?,
        Ok(_) => fs::remove_file(&cache)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

/// Makes `path` and each folder on the way below `HOME`, owned by `ada`.
pub fn owned_folders(path: &Path, uid: u32, gid: u32) -> TestResult {
    let mut folder = PathBuf::from(HOME);

    for part in path.strip_prefix(HOME)?.components() {
        folder.push(part);
        fs::create_dir_all(&folder)?;
        chown(&folder, Some(uid), Some(gid))?;
    }

    Ok(())
}

fn pkgbuild(extra: &str) -> String {
    format!(
        "pkgname={PACKAGE}\npkgver=1.0\npkgrel=1\npkgdesc=\"Garuda Toolbox integration test package\"\narch=(any)\nlicense=(GPL-3.0-or-later)\noptions=(!debug)\n{extra}\npackage() {{\n  install -Dm644 /dev/null \"$pkgdir/usr/share/toolbox-test/{PACKAGE}\"\n}}\n"
    )
}

/// Builds the package as `ada`, with the arguments the app gives makepkg,
/// into the run folder `run`. `extra` is one more line of the PKGBUILD.
pub fn build(run: &str, extra: &str) -> TestResult {
    let (uid, gid) = account()?;
    fresh_cache()?;

    let clone = Path::new(HOME).join(AUR_CLONE).join(PACKAGE);
    owned_folders(&clone, uid, gid)?;

    let script = clone.join("PKGBUILD");
    fs::write(&script, pkgbuild(extra))?;
    chown(&script, Some(uid), Some(gid))?;

    let folder = run_folder(run);
    owned_folders(&folder, uid, gid)?;
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o700))?;

    let output = Command::new("/usr/bin/makepkg")
        .args(MAKEPKG_ARGS)
        .current_dir(&clone)
        .env_clear()
        .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/bin")
        .env("HOME", HOME)
        .env("PKGDEST", &folder)
        .uid(uid)
        .gid(gid)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "makepkg failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    )
    .into())
}

pub fn explicit(name: &str) -> bool {
    listed("-Qeq", name)
}

pub fn dependency(name: &str) -> bool {
    listed("-Qdq", name)
}

pub fn foreign(name: &str) -> bool {
    listed("-Qmq", name)
}

fn listed(flag: &str, name: &str) -> bool {
    Command::new("pacman")
        .arg(flag)
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line == name)
        })
}

/// Whether the root store holds nothing, its folder aside.
pub fn store_is_empty() -> Result<bool, Box<dyn Error>> {
    match fs::read_dir(Path::new("/").join(AUR_STORE)) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error.into()),
    }
}

/// The helper with stdin a socket whose peer is `uid`, the way the app runs
/// it for a person. `at_gate` answers each gate.
pub fn session(
    uid: u32,
    text: &str,
    at_gate: impl FnMut(StepId) -> Option<Answer>,
) -> Result<Run, Box<dyn Error>> {
    let (app, helper) = person::socket_of(uid)?;
    let child = person::command(&["apply"], false)
        .stdin(Stdio::from(helper))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    drive(child, app, text, None, at_gate)
}

/// The helper with stdin a pipe, which names no caller.
pub fn piped(
    text: &str,
    at_gate: impl FnMut(StepId) -> Option<Answer>,
) -> Result<Run, Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"));

    command.arg("apply");
    run_helper_at(command, text, None, at_gate)
}

/// The outcomes and the printed text of `actions` run as `uid`.
pub fn as_uid(
    uid: u32,
    actions: Vec<RootAction>,
) -> Result<(Vec<Outcome>, String), Box<dyn Error>> {
    let output = session(uid, &request(Mode::Run, actions)?, |step| {
        Some(Answer::Proceed { step })
    })?;

    Ok((outcomes(&output)?, printed(&output)?))
}

/// The outcomes and the printed text of `actions` run as `ada`.
pub fn as_ada(actions: Vec<RootAction>) -> Result<(Vec<Outcome>, String), Box<dyn Error>> {
    as_uid(account()?.0, actions)
}

/// `InstallBuilt` of the package in `run`, asked for by `uid`.
pub fn install(
    uid: u32,
    run: &str,
    reason: InstallReason,
) -> Result<(Vec<Outcome>, String), Box<dyn Error>> {
    as_uid(
        uid,
        vec![RootAction::InstallBuilt {
            run: BuildRun::new(run)?,
            packages: packages(&[PACKAGE])?,
            reason,
        }],
    )
}

fn pacman_install(name: &str) -> TestResult {
    Command::new("pacman")
        .args(["-S", "--noconfirm", "--", name])
        .status()?;

    Ok(())
}

#[test]
fn install_dependencies_installs_the_unmet_names_and_keeps_an_explicit_one() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    pacman_install("toolbox-test-dep")?;
    assert!(explicit("toolbox-test-dep"));

    let results = apply(vec![RootAction::InstallDependencies {
        packages: packages(&["toolbox-test-a", "toolbox-test-dep"])?,
    }])?;

    assert_eq!(results, [Outcome::Succeeded]);
    assert!(
        dependency("toolbox-test-a"),
        "an unmet name comes in as a dependency"
    );
    assert!(
        explicit("toolbox-test-dep"),
        "an installed name keeps its reason"
    );
    Ok(())
}

#[test]
fn a_built_package_is_installed_as_explicit_and_the_store_is_left_empty() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    build("explicit", "")?;

    let (results, text) = install(account()?.0, "explicit", InstallReason::Explicit)?;

    assert_eq!(results, [Outcome::Succeeded], "{text}");
    assert!(installed(PACKAGE) && explicit(PACKAGE) && foreign(PACKAGE));
    assert!(store_is_empty()?);
    Ok(())
}

#[test]
fn a_built_package_is_installed_as_dependency() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    build("dependency", "")?;

    let (results, text) = install(account()?.0, "dependency", InstallReason::Dependency)?;

    assert_eq!(results, [Outcome::Succeeded], "{text}");
    assert!(dependency(PACKAGE));
    Ok(())
}

/// An upgrade with `Keep` leaves the reason pacman has for the package. The
/// reinstall of the same version stands in for an upgrade here.
#[test]
fn keep_keeps_the_reason_an_installed_package_has() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    build("first", "")?;
    let (first, first_text) = install(account()?.0, "first", InstallReason::Dependency)?;
    assert_eq!(first, [Outcome::Succeeded], "{first_text}");

    build("second", "")?;
    let (second, second_text) = install(account()?.0, "second", InstallReason::Keep)?;

    assert_eq!(second, [Outcome::Succeeded], "{second_text}");
    assert!(
        dependency(PACKAGE),
        "Keep passes no reason, so pacman keeps the dependency"
    );
    Ok(())
}

/// The removal after a build: a make dependency that `InstallDependencies`
/// brought in, and that nothing needs, goes in the same session as the install.
#[test]
fn a_make_dependency_of_a_build_is_removed_after_it() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    let (before, before_text) = as_ada(vec![RootAction::InstallDependencies {
        packages: packages(&["toolbox-test-dep"])?,
    }])?;

    assert_eq!(before, [Outcome::Succeeded], "{before_text}");

    build("chain", "makedepends=(toolbox-test-dep)")?;

    let (results, chain_text) = as_ada(vec![
        RootAction::InstallBuilt {
            run: BuildRun::new("chain")?,
            packages: packages(&[PACKAGE])?,
            reason: InstallReason::Explicit,
        },
        RootAction::RemoveMakeDependencies {
            packages: packages(&["toolbox-test-dep"])?,
        },
    ])?;

    assert_eq!(
        results,
        [Outcome::Succeeded, Outcome::Succeeded],
        "{chain_text}"
    );
    assert!(installed(PACKAGE) && !installed("toolbox-test-dep"));
    Ok(())
}
