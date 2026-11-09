//! A kernel entry, as the Setup Assistant lists them, meets the guards of
//! the Kernels page.

use toolbox_catalog::ToggleCatalog;
use toolbox_state::kernels::{LinuxPackages, LocalPackage, RunningKernel};
use toolbox_state::packages::PackageGraph;

use super::super::{Refusal, toggle};
use crate::domains::{InstalledPackages, Packages};
use crate::features::kernels;
use crate::testing::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

const KERNELS: &str = r#"
[[section]]
id = "kernels"
title = "Kernels"
icon = "x"

[[section.entry]]
id = "linux-lts"
title = "LTS kernel"
description = "L"
check = { package = "linux-lts" }
also = [{ package = "linux-lts-headers" }]
"#;

/// `linux-lts` is the only kernel and runs.
fn only_lts() -> Snapshot {
    let names = ["linux-lts", "linux-lts-headers"];
    let mut snapshot = Snapshot::default();
    snapshot.insert(LinuxPackages {
        sync: Vec::new(),
        local: names
            .iter()
            .map(|name| LocalPackage {
                name: package(name),
                version: "6.12.9-1".to_owned(),
                description: String::new(),
            })
            .collect(),
    });
    snapshot.insert(InstalledPackages(packages(&names)));
    snapshot.insert(RunningKernel {
        release: "6.12.9-1-lts".to_owned(),
        package: Some(package("linux-lts")),
    });
    snapshot.insert(PackageGraph::default());
    snapshot
}

#[test]
fn the_running_kernel_entry_cannot_be_turned_off() {
    let catalog = ToggleCatalog::parse(KERNELS, "setup.toml").unwrap();
    let mut changes = ChangeSet::default();

    let refused = toggle(
        &catalog,
        "linux-lts",
        Toggle::Off,
        &only_lts(),
        &mut changes,
    );

    assert_eq!(
        refused,
        Err(Refusal::Kernel(
            kernels::Refusal::Running,
            package("linux-lts")
        ))
    );
    assert_eq!(changes.wish::<Packages>(&package("linux-lts")), None);
    assert_eq!(
        changes.wish::<Packages>(&package("linux-lts-headers")),
        None
    );
}

#[test]
fn a_kernel_refusal_names_the_kernel_with_the_kernels_page_code() {
    let refusal = Refusal::Kernel(kernels::Refusal::LastKernel, package("linux-lts"));

    assert_eq!(refusal.as_str(), "last-kernel");
    assert_eq!(refusal.name(), "linux-lts");
}
