use std::sync::Arc;
use std::time::{Duration, SystemTime};

use toolbox_plan::{PackageName, UnitName};
use toolbox_state::parts::{InstalledPackages, SyncPackages};

use super::{DiskSpace, FactSources, MachineSource, PackageCount, SystemMachine, probe_facts};
use crate::ProbeError;
use crate::packages::{OrphanSource, PackageSource, SyncSource};
use crate::testing::write;
use crate::units::FailedUnitSource;

/// The Lima VM's running kernel and its `/proc/stat`, whose `btime` is
/// [`BOOTED`].
const OSRELEASE: &str = include_str!("../kernels/fixtures/image/osrelease");
const STAT: &str = include_str!("../host/fixtures/proc-stat");
const BOOTED: u64 = 1_794_155_377;
/// `systemctl list-units --state=failed` in the VM.
const FAILED: &str = include_str!("../units/fixtures/failed");

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
        Ok(OSRELEASE.trim().to_owned())
    }

    fn booted(&self) -> Result<SystemTime, ProbeError> {
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(BOOTED))
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

/// The packages of the VM's local database fixture plus `cbonsai`, which
/// `paru -Qua` there listed from the AUR. The unit is the VM's first
/// failed one.
fn sources(offered: Result<SyncPackages, ProbeError>) -> FactSources {
    let failed = FAILED.split_whitespace().next().unwrap();

    FactSources {
        installed: Arc::new(FakeInstalled(Ok(InstalledPackages(packages(&[
            "cbonsai", "gegl", "gimp", "mpv", "yt-dlp",
        ]))))),
        offered: Arc::new(FakeOffered(offered)),
        orphans: Arc::new(FakeOrphans(vec![PackageName::new("gegl").unwrap()])),
        machine: Arc::new(FakeMachine),
        units: Arc::new(FakeUnits(vec![UnitName::new(failed).unwrap()])),
    }
}

#[test]
fn installed_packages_in_no_repository_count_as_foreign() {
    let facts = probe_facts(&sources(Ok(SyncPackages(packages(&[
        "gegl", "gimp", "mpv", "yt-dlp",
    ])))));

    assert_eq!(
        facts.packages,
        Ok(PackageCount {
            installed: 5,
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
            installed: 5,
            foreign: None,
        })
    );
}

#[test]
fn one_failed_fact_leaves_the_others() {
    let facts = probe_facts(&sources(Err(ProbeError::Unavailable)));

    assert_eq!(facts.kernel.as_deref(), Ok("7.3.1-1-garuda"));
    facts.disk.unwrap_err();
    assert_eq!(facts.package_cache, Err(ProbeError::Unavailable));
    assert_eq!(facts.orphans.map(|orphans| orphans.len()), Ok(1));
    assert_eq!(facts.failed_units.map(|units| units.len()), Ok(1));
}

#[test]
fn the_machine_reads_kernel_and_boot_time_below_its_root() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "proc/sys/kernel/osrelease", OSRELEASE);
    write(root.path(), "proc/stat", STAT);
    let machine = SystemMachine::below(root.path().to_owned());

    assert_eq!(machine.kernel().as_deref(), Ok("7.3.1-1-garuda"));
    assert_eq!(
        machine.booted(),
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(BOOTED))
    );
}

#[test]
fn the_package_cache_counts_its_files_only() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "var/cache/pacman/pkg/gimp-3.2.6-2-x86_64.pkg.tar.zst",
        [0_u8; 300],
    );
    write(
        root.path(),
        "var/cache/pacman/pkg/gimp-3.2.6-2-x86_64.pkg.tar.zst.sig",
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
