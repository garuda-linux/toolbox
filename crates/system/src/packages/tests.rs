mod fixture;
mod index;
mod orphans;

use std::fs;

use fixture::{GEGL, GIMP, MPV, System, YT_DLP};
use toolbox_plan::PackageName;
use toolbox_state::parts::{InstalledPackages, SyncPackages};

use super::{Libalpm, LinuxPackageSource as _, PackageSource as _, SyncSource as _};
use crate::ProbeError;

fn names(names: &[&str]) -> std::collections::BTreeSet<PackageName> {
    names
        .iter()
        .map(|name| PackageName::new(*name).unwrap())
        .collect()
}

#[test]
fn the_local_database_lists_the_installed_packages() {
    let system = System::new(&["core"]);

    system.install_captured(&[GIMP, GEGL, MPV, YT_DLP]);

    assert_eq!(
        Libalpm::with_config(system.config()).installed(),
        Ok(InstalledPackages(names(&["gegl", "gimp", "mpv", "yt-dlp"])))
    );
}

#[test]
fn the_sync_databases_list_what_the_repos_offer() {
    let system = System::new(&["core", "extra", "never-synced"]);

    system.install(&[]);
    system.sync("core", &["bash", "pacman"]);
    system.sync("extra", &["steam", "bash"]);

    assert_eq!(
        Libalpm::with_config(system.config()).offered(),
        Ok(SyncPackages(names(&["bash", "pacman", "steam"])))
    );
}

#[test]
fn without_any_synced_database_the_offer_is_unavailable() {
    let system = System::new(&["core"]);

    system.install(&["bash"]);

    assert_eq!(
        Libalpm::with_config(system.config()).offered(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn without_a_local_database_nothing_is_known_to_be_installed() {
    let system = System::new(&["core"]);

    assert_eq!(
        Libalpm::with_config(system.config()).installed(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn without_pacman_conf_both_are_unavailable() {
    let alpm = Libalpm::with_config("/toolbox-system/no/pacman.conf");

    assert_eq!(alpm.installed(), Err(ProbeError::Unavailable));
    assert_eq!(alpm.offered(), Err(ProbeError::Unavailable));
}

/// A local database of another layout version lists nothing, which must
/// not read as "nothing installed". The captured `ALPM_DB_VERSION` 9 is
/// overwritten with 1.
#[test]
fn an_unreadable_local_database_is_a_failure() {
    let system = System::new(&["core"]);

    system.install_captured(&[GIMP]);
    fs::write(system.path().join("db/local/ALPM_DB_VERSION"), "1\n").unwrap();

    assert!(matches!(
        Libalpm::with_config(system.config()).installed(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn linux_packages_come_with_repository_version_and_description() {
    let system = System::new(&["garuda", "core"]);

    system.install_versions(&[
        ("bash", "5.2-1"),
        ("linux", "6.14.0-3"),
        ("linux-firmware", "20250311-1"),
    ]);
    system.sync_versions(
        "core",
        &[
            ("linux-headers", "6.14.0-4"),
            ("linux", "6.14.0-4"),
            ("bash", "5.2-1"),
        ],
    );
    system.sync_versions(
        "garuda",
        &[("linux-zen", "6.13.8.zen1-1"), ("alinux", "1-1")],
    );

    let packages = Libalpm::with_config(system.config())
        .linux_packages()
        .unwrap();

    let sync: Vec<(&str, &str, &str, &str)> = packages
        .sync
        .iter()
        .map(|package| {
            (
                package.repo.as_str(),
                package.name.as_str(),
                package.version.as_str(),
                package.description.as_str(),
            )
        })
        .collect();

    let local: Vec<(&str, &str, &str)> = packages
        .local
        .iter()
        .map(|package| {
            (
                package.name.as_str(),
                package.version.as_str(),
                package.description.as_str(),
            )
        })
        .collect();

    assert_eq!(
        sync,
        [
            ("garuda", "linux-zen", "6.13.8.zen1-1", "About linux-zen"),
            ("core", "linux", "6.14.0-4", "About linux"),
            ("core", "linux-headers", "6.14.0-4", "About linux-headers"),
        ]
    );

    assert_eq!(
        local,
        [
            ("linux", "6.14.0-3", "About linux"),
            ("linux-firmware", "20250311-1", "About linux-firmware"),
        ]
    );
}

/// Installed kernels show even when the databases were never synced.
#[test]
fn without_synced_databases_only_installed_linux_packages_are_listed() {
    let system = System::new(&["core"]);

    system.install_versions(&[("linux-lts", "6.12.9-1")]);

    let packages = Libalpm::with_config(system.config())
        .linux_packages()
        .unwrap();

    assert_eq!(packages.sync, []);
    assert_eq!(packages.local.len(), 1);
}

#[test]
fn without_a_local_database_linux_packages_are_unavailable() {
    let system = System::new(&["core"]);

    system.sync("core", &["linux"]);

    assert_eq!(
        Libalpm::with_config(system.config()).linux_packages(),
        Err(ProbeError::Unavailable)
    );
}

/// The databases are unpacked once and again only after a new download.
#[test]
fn the_sync_read_is_reused_until_a_database_file_changes() {
    let system = System::new(&["core"]);
    let alpm = Libalpm::with_config(system.config());

    system.install(&[]);
    system.sync("core", &["bash"]);
    assert_eq!(alpm.offered(), Ok(SyncPackages(names(&["bash"]))));

    system.sync("core", &["bash", "linux", "linux-headers"]);
    assert_eq!(
        alpm.offered(),
        Ok(SyncPackages(names(&["bash", "linux", "linux-headers"])))
    );
    assert_eq!(alpm.linux_packages().unwrap().sync.len(), 2);
}
