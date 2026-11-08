use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_plan::{KernelRelease, PackageName};
use toolbox_state::kernels::{DkmsState, KernelReleases, RunningKernel};

use super::{Dkms, DkmsSource as _, KernelSource as _, SystemKernels};
use crate::ProbeError;
use crate::testing::fake_program;

fn modules(root: &Path, release: &str, pkgbase: Option<&str>) {
    let folder = root.join("usr/lib/modules").join(release);

    fs::create_dir_all(&folder).unwrap();

    if let Some(package) = pkgbase {
        fs::write(folder.join("pkgbase"), format!("{package}\n")).unwrap();
    }
}

fn booted(release: &str) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    let proc = root.path().join("proc/sys/kernel");

    fs::create_dir_all(&proc).unwrap();
    fs::write(proc.join("osrelease"), format!("{release}\n")).unwrap();
    root
}

#[test]
fn the_modules_folder_names_the_package_of_the_running_kernel() {
    let root = booted("6.13.8-zen1-1-zen");
    modules(root.path(), "6.13.8-zen1-1-zen", Some("linux-zen"));

    assert_eq!(
        SystemKernels::new(root.path()).running(),
        Ok(RunningKernel {
            release: "6.13.8-zen1-1-zen".to_owned(),
            package: Some(PackageName::new("linux-zen").unwrap()),
        })
    );
}

/// After a kernel update the booted release's folder is gone until the
/// next boot.
#[test]
fn a_running_kernel_without_its_folder_has_no_package() {
    let root = booted("6.13.7-zen1-1-zen");
    modules(root.path(), "6.13.8-zen1-1-zen", Some("linux-zen"));

    assert_eq!(
        SystemKernels::new(root.path()).running(),
        Ok(RunningKernel {
            release: "6.13.7-zen1-1-zen".to_owned(),
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

#[test]
fn each_kernel_package_gets_its_release() {
    let root = tempfile::tempdir().unwrap();
    modules(root.path(), "6.14.0-arch1-1", Some("linux"));
    modules(root.path(), "6.12.9-1-lts", Some("linux-lts"));
    modules(root.path(), "6.12.8-1-lts", Some("linux-lts"));
    modules(root.path(), "6.11.0-arch1-1", None);
    modules(root.path(), "extramodules-6.14", Some("not a package"));
    fs::write(root.path().join("usr/lib/modules/a-file"), "").unwrap();

    assert_eq!(
        SystemKernels::new(root.path()).releases(),
        Ok(KernelReleases(BTreeMap::from([
            (
                PackageName::new("linux").unwrap(),
                KernelRelease::new("6.14.0-arch1-1").unwrap()
            ),
            (
                PackageName::new("linux-lts").unwrap(),
                KernelRelease::new("6.12.9-1-lts").unwrap()
            ),
        ])))
    );
}

/// As text `6.9.10` sorts before `6.9.9`.
#[test]
fn of_two_folders_of_one_package_the_newer_version_counts() {
    let root = tempfile::tempdir().unwrap();
    modules(root.path(), "6.9.9-arch1-1", Some("linux"));
    modules(root.path(), "6.9.10-arch1-1", Some("linux"));
    modules(root.path(), "6.10.2-zen1-1-zen", Some("linux-zen"));
    modules(root.path(), "6.9.12-zen1-1-zen", Some("linux-zen"));

    assert_eq!(
        SystemKernels::new(root.path()).releases(),
        Ok(KernelReleases(BTreeMap::from([
            (
                PackageName::new("linux").unwrap(),
                KernelRelease::new("6.9.10-arch1-1").unwrap()
            ),
            (
                PackageName::new("linux-zen").unwrap(),
                KernelRelease::new("6.10.2-zen1-1-zen").unwrap()
            ),
        ])))
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
    let (_folder, program) = fake_program(
        "nvidia/550.90.07, 6.14.0-arch1-1, x86_64: installed\nzfs/2.3.1: added\n",
        "",
        0,
    );

    let entries = Dkms::with_program(&program).status().unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].module.as_str(), "nvidia");
    assert_eq!(entries[0].state, DkmsState::Installed);
    assert_eq!(entries[1].kernel, None);
}

#[test]
fn a_failing_dkms_that_still_lists_modules_counts() {
    let (_folder, program) = fake_program(
        "vboxhost/7.1.6_OSE, 6.12.9-1-lts, x86_64: broken\n",
        "Error! vboxhost/7.1.6_OSE: Missing the module source directory",
        4,
    );

    assert_eq!(
        Dkms::with_program(&program).status().unwrap()[0].state,
        DkmsState::Broken
    );
}

#[test]
fn a_failing_dkms_without_modules_is_an_error() {
    let (_folder, program) = fake_program("", "dkms: permission denied", 1);

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
