use std::sync::Arc;
use std::time::{Duration, SystemTime};

use toolbox_plan::{PackageName, UnitName};
use toolbox_state::parts::{InstalledPackages, SyncPackages};

use super::{DiskSpace, FactSources, MachineSource, PackageCount, SystemMachine, probe_facts};
use crate::ProbeError;
use crate::packages::{OrphanSource, PackageSource, SyncSource};
use crate::testing::write;
use crate::units::FailedUnitSource;

struct FakeInstalled(Result<InstalledPackages, ProbeError>);

struct FakeOffered(Result<SyncPackages, ProbeError>);

struct FakeOrphans(Vec<PackageName>);

struct FakeMachine;

struct FakeUnits(Vec<UnitName>);

impl PackageSource for FakeInstalled {
    fn installed(&self) -> Result<InstalledPackages, ProbeError> {
        self.0.clone()
    }
}

impl SyncSource for FakeOffered {
    fn offered(&self) -> Result<SyncPackages, ProbeError> {
        self.0.clone()
    }
}

impl OrphanSource for FakeOrphans {
    fn orphans(&self) -> Result<Vec<PackageName>, ProbeError> {
        Ok(self.0.clone())
    }
}

impl MachineSource for FakeMachine {
    fn kernel(&self) -> Result<String, ProbeError> {
        Ok("6.17.7-zen1-1-zen".to_owned())
    }

    fn booted(&self) -> Result<SystemTime, ProbeError> {
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(1_000))
    }

    fn disk(&self) -> Result<DiskSpace, ProbeError> {
        Err(ProbeError::Failed("statvfs: denied".to_owned()))
    }

    fn package_cache(&self) -> Result<u64, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

impl FailedUnitSource for FakeUnits {
    fn failed_units(&self) -> Result<Vec<UnitName>, ProbeError> {
        Ok(self.0.clone())
    }
}

fn packages(names: &[&str]) -> std::collections::BTreeSet<PackageName> {
    names
        .iter()
        .map(|name| PackageName::new(*name).unwrap())
        .collect()
}

fn sources(offered: Result<SyncPackages, ProbeError>) -> FactSources {
    FactSources {
        installed: Arc::new(FakeInstalled(Ok(InstalledPackages(packages(&[
            "bash",
            "paru",
            "zen-browser-bin",
        ]))))),
        offered: Arc::new(FakeOffered(offered)),
        orphans: Arc::new(FakeOrphans(vec![PackageName::new("old-lib").unwrap()])),
        machine: Arc::new(FakeMachine),
        units: Arc::new(FakeUnits(vec![UnitName::new("smb.service").unwrap()])),
    }
}

#[test]
fn installed_packages_in_no_repository_count_as_foreign() {
    let facts = probe_facts(&sources(Ok(SyncPackages(packages(&["bash", "paru"])))));

    assert_eq!(
        facts.packages,
        Ok(PackageCount {
            installed: 3,
            foreign: Some(1),
        })
    );
}

#[test]
fn without_a_synced_database_the_foreign_count_is_unknown() {
    let facts = probe_facts(&sources(Err(ProbeError::Unavailable)));

    assert_eq!(
        facts.packages,
        Ok(PackageCount {
            installed: 3,
            foreign: None,
        })
    );
}

#[test]
fn one_failed_fact_leaves_the_others() {
    let facts = probe_facts(&sources(Err(ProbeError::Unavailable)));

    assert_eq!(facts.kernel.as_deref(), Ok("6.17.7-zen1-1-zen"));
    facts.disk.unwrap_err();
    assert_eq!(facts.package_cache, Err(ProbeError::Unavailable));
    assert_eq!(facts.orphans.map(|orphans| orphans.len()), Ok(1));
    assert_eq!(facts.failed_units.map(|units| units.len()), Ok(1));
}

#[test]
fn the_machine_reads_kernel_and_boot_time_below_its_root() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "proc/sys/kernel/osrelease",
        "6.17.7-zen1-1-zen\n",
    );
    write(root.path(), "proc/stat", "cpu 1 2 3\nbtime 1700000000\n");
    let machine = SystemMachine::below(root.path().to_owned());

    assert_eq!(machine.kernel().as_deref(), Ok("6.17.7-zen1-1-zen"));
    assert_eq!(
        machine.booted(),
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000))
    );
}

#[test]
fn the_package_cache_counts_its_files_only() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "var/cache/pacman/pkg/bash-5.2-1.pkg.tar.zst",
        [0_u8; 300],
    );
    write(
        root.path(),
        "var/cache/pacman/pkg/bash-5.2-1.pkg.tar.zst.sig",
        [0_u8; 20],
    );
    write(
        root.path(),
        "var/cache/pacman/pkg/download-x/part",
        [0_u8; 999],
    );
    let machine = SystemMachine::below(root.path().to_owned());

    assert_eq!(machine.package_cache(), Ok(320));
}

#[test]
fn a_missing_package_cache_is_unavailable() {
    let root = tempfile::tempdir().unwrap();
    let machine = SystemMachine::below(root.path().to_owned());

    assert_eq!(machine.package_cache(), Err(ProbeError::Unavailable));
    assert_eq!(machine.kernel(), Err(ProbeError::Unavailable));
}

#[test]
fn the_disk_of_a_real_folder_has_room() {
    let root = tempfile::tempdir().unwrap();
    let space = SystemMachine::below(root.path().to_owned()).disk().unwrap();

    assert!(space.total > 0);
    assert!(space.free <= space.total);
}
