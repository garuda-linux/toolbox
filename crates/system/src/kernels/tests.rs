use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_plan::{KernelRelease, PackageName};
use toolbox_state::kernels::{DkmsState, KernelReleases, RunningKernel};

use super::{Dkms, DkmsSource as _, KernelSource as _, SystemKernels};
use crate::ProbeError;
use crate::testing::fake_program;

/// The Lima VM with `linux-garuda` only.
const IMAGE_RELEASE: &str = include_str!("fixtures/image/osrelease");
const IMAGE_PKGBASE: &str = include_str!("fixtures/image/pkgbase");
/// The Lima VM after `linux-lts` was installed next to it.
const TWO_RELEASE: &str = include_str!("fixtures/two/osrelease");
const TWO_PKGBASE: &str = include_str!("fixtures/two/pkgbase");

const DKMS_INSTALLED: &str =
    include_str!("../../../state/src/kernels/dkms/fixtures/installed.stdout");
const DKMS_ADDED: &str = include_str!("../../../state/src/kernels/dkms/fixtures/added.stdout");
const DKMS_BROKEN: &str = include_str!("../../../state/src/kernels/dkms/fixtures/broken.stdout");
const DKMS_BROKEN_ERR: &str =
    include_str!("../../../state/src/kernels/dkms/fixtures/broken.stderr");

/// The `usr/lib/modules` folders of a capture with lines of
/// `<folder>\t<pkgbase>`. An empty pkgbase writes no file.
fn modules(root: &Path, pkgbase: &str) {
    for line in pkgbase.lines() {
        let (release, package) = line.split_once('\t').unwrap();
        let folder = root.join("usr/lib/modules").join(release);

        fs::create_dir_all(&folder).unwrap();

        if !package.is_empty() {
            fs::write(folder.join("pkgbase"), package).unwrap();
        }
    }
}

fn booted(release: &str) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    let proc = root.path().join("proc/sys/kernel");

    fs::create_dir_all(&proc).unwrap();
    fs::write(proc.join("osrelease"), release).unwrap();
    root
}

fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap()
}

fn release(name: &str) -> KernelRelease {
    KernelRelease::new(name).unwrap()
}

fn two_releases() -> KernelReleases {
    KernelReleases(BTreeMap::from([
        (package("linux-garuda"), release("7.3.1-1-garuda")),
        (package("linux-lts"), release("6.18.60-1-lts")),
    ]))
}

#[test]
fn the_modules_folder_names_the_package_of_the_running_kernel() {
    let root = booted(IMAGE_RELEASE);
    modules(root.path(), IMAGE_PKGBASE);

    assert_eq!(
        SystemKernels::new(root.path()).running(),
        Ok(RunningKernel {
            release: "7.3.1-1-garuda".to_owned(),
            package: Some(package("linux-garuda")),
        })
    );
}

/// After a kernel update the booted release's folder is gone until the
/// next boot. The two-kernel VM keeps only its `linux-lts` folder here.
#[test]
fn a_running_kernel_without_its_folder_has_no_package() {
    let root = booted(TWO_RELEASE);
    let lts = TWO_PKGBASE
        .lines()
        .find(|line| line.contains("lts"))
        .unwrap();

    modules(root.path(), lts);

    assert_eq!(
        SystemKernels::new(root.path()).running(),
        Ok(RunningKernel {
            release: "7.3.1-1-garuda".to_owned(),
            package: None,
        })
    );
}

#[test]
fn without_proc_the_running_kernel_is_unavailable() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        SystemKernels::new(root.path()).running(),
        Err(ProbeError::Unavailable)
    );
}

/// The two-kernel VM plus edits: an older `linux-lts` leftover, a folder
/// without `pkgbase`, one whose `pkgbase` is no package name, and a file.
#[test]
fn each_kernel_package_gets_its_release() {
    let root = tempfile::tempdir().unwrap();
    let edited = format!(
        "{TWO_PKGBASE}{}\n6.17.9-1-garuda\t\nextramodules-7.3\tnot a package\n",
        TWO_PKGBASE
            .lines()
            .next()
            .unwrap()
            .replacen("6.18.60", "6.18.59", 1)
    );

    modules(root.path(), &edited);
    fs::write(root.path().join("usr/lib/modules/a-file"), "").unwrap();

    assert_eq!(
        SystemKernels::new(root.path()).releases(),
        Ok(two_releases())
    );
}

/// As text `6.18.60` sorts before `6.18.9`. The test adds a `6.18.9`
/// leftover next to the captured `6.18.60` folder of `linux-lts`.
#[test]
fn of_two_folders_of_one_package_the_newer_version_counts() {
    let root = tempfile::tempdir().unwrap();
    let lts = TWO_PKGBASE.lines().next().unwrap();
    modules(
        root.path(),
        &format!("{TWO_PKGBASE}{}\n", lts.replacen("6.18.60", "6.18.9", 1)),
    );

    assert_eq!(
        SystemKernels::new(root.path()).releases(),
        Ok(two_releases())
    );
}

#[test]
fn without_a_modules_folder_the_releases_are_unavailable() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        SystemKernels::new(root.path()).releases(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn dkms_status_is_parsed_from_the_program() {
    let (_installed_folder, installed) = fake_program(DKMS_INSTALLED, "", 0);
    let (_added_folder, added) = fake_program(DKMS_ADDED, "", 0);

    let entries = Dkms::with_program(&installed).status().unwrap();
    let added = Dkms::with_program(&added).status().unwrap();

    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].module.as_str(), "broadcom-wl");
    assert_eq!(entries[0].state, DkmsState::Installed);
    assert_eq!(entries[3].state, DkmsState::Installed);
    assert_eq!(added[2].module.as_str(), "v4l2loopback");
    assert_eq!(added[2].kernel, None);
    assert_eq!(added[2].state, DkmsState::Added);
}

/// The captured dkms exits with 0 for a broken module. The same capture
/// with a failing exit code must still count.
#[test]
fn a_dkms_that_complains_but_still_lists_modules_counts() {
    for code in [0, 4] {
        let (_folder, program) = fake_program(DKMS_BROKEN, DKMS_BROKEN_ERR, code);

        assert_eq!(
            Dkms::with_program(&program).status().unwrap()[2].state,
            DkmsState::Broken,
            "exit code {code}"
        );
    }
}

/// The broken capture's complaint without its list and with a failing
/// exit code.
#[test]
fn a_failing_dkms_without_modules_is_an_error() {
    let (_folder, program) = fake_program("", DKMS_BROKEN_ERR, 1);

    assert!(matches!(
        Dkms::with_program(&program).status(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn no_modules_and_success_is_an_empty_list() {
    let (_folder, program) = fake_program("", "", 0);

    assert_eq!(Dkms::with_program(&program).status(), Ok(Vec::new()));
}

#[test]
fn a_system_without_dkms_is_unavailable() {
    assert_eq!(
        Dkms::with_program("/nonexistent/dkms").status(),
        Err(ProbeError::Unavailable)
    );
}
