//! The guards of the page, applied to a queue restored from disk.

use serde_json::{Value, json};

use super::super::{LinuxPackages, RunningKernel, recheck};
use super::{local, repo, wish};
use crate::domains::InstalledPackages;
use crate::testing::{package, packages, restore_saved};
use crate::{ChangeSet, Snapshot, Toggle};

/// `linux` runs, `linux-lts` is installed beside it, both with headers.
fn system() -> Snapshot {
    let names = ["linux", "linux-headers", "linux-lts", "linux-lts-headers"];
    let mut snapshot = Snapshot::default();

    snapshot.insert(LinuxPackages {
        sync: names
            .iter()
            .map(|name| repo("core", name, "6.14.0-4"))
            .collect(),
        local: names.iter().map(|name| local(name, "6.14.0-4")).collect(),
    });
    snapshot.insert(InstalledPackages(packages(&names)));
    snapshot.insert(RunningKernel {
        release: "6.14.0-4-arch".to_owned(),
        package: Some(package("linux")),
    });
    snapshot
}

fn restored_removals(names: &[&str]) -> ChangeSet {
    let saved: Vec<(&str, Value, Value)> = names
        .iter()
        .map(|name| ("packages", json!(name), json!("off")))
        .collect();

    restore_saved(&saved)
}

#[test]
fn a_restored_removal_of_the_running_kernel_is_dropped_with_its_headers() {
    let mut changes = restored_removals(&["linux", "linux-headers", "linux-lts", "vlc"]);

    assert_eq!(recheck(&mut changes, &system()), Some(1));
    assert_eq!(wish(&changes, "linux"), None);
    assert_eq!(wish(&changes, "linux-headers"), None);
    assert_eq!(wish(&changes, "linux-lts"), Some(Toggle::Off));
    assert_eq!(wish(&changes, "vlc"), Some(Toggle::Off));
}

#[test]
fn without_the_running_kernel_every_restored_kernel_removal_is_dropped() {
    let mut snapshot = system();

    snapshot.insert(RunningKernel::default());

    let mut changes = restored_removals(&["linux-lts", "linux-lts-headers", "vlc"]);

    assert_eq!(recheck(&mut changes, &snapshot), Some(1));
    assert_eq!(wish(&changes, "linux-lts"), None);
    assert_eq!(wish(&changes, "linux-lts-headers"), None);
    assert_eq!(wish(&changes, "vlc"), Some(Toggle::Off));
}

#[test]
fn without_the_kernel_packages_the_restored_queue_stays() {
    let mut changes = restored_removals(&["linux"]);

    assert_eq!(recheck(&mut changes, &Snapshot::default()), None);
    assert_eq!(changes.len(), 1);
}
