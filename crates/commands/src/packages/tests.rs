use std::collections::BTreeSet;

use toolbox_cli::args::{Confirm, PackageArgs, PackagesSearch, Read};
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, Origin, PackageDetails, PackageIndex,
};
use toolbox_core::{Snapshot, Toggle};
use toolbox_plan::{PackageName, PlannedAction, RootAction};
use toolbox_system::{PackageIndexSource, ProbeError};

use super::{PackageInfo, change, found, info_text, listing};
use crate::change::Refused;

/// gimp installed with its leftover dependency, inkscape on offer.
struct Index;

impl PackageIndexSource for Index {
    fn index(&self) -> Result<PackageIndex, ProbeError> {
        let entry = |text: &str, installed: Option<InstallReason>| IndexEntry {
            name: name(text),
            description: format!("About {text}"),
            version: "1.0-1".to_owned(),
            origin: Origin::Repository("extra".to_owned()),
            size: 2048,
            installed: installed.map(|reason| Installed {
                version: "1.0-1".to_owned(),
                reason,
            }),
        };

        Ok(PackageIndex {
            entries: vec![
                entry("gimp", Some(InstallReason::Explicit)),
                entry("inkscape", None),
                entry("leftover", Some(InstallReason::Dependency)),
            ],
            local: vec![
                LocalNode {
                    name: name("gimp"),
                    depends: Vec::new(),
                    provides: Vec::new(),
                    reason: InstallReason::Explicit,
                },
                LocalNode {
                    name: name("leftover"),
                    depends: Vec::new(),
                    provides: Vec::new(),
                    reason: InstallReason::Dependency,
                },
            ],
        })
    }

    fn details(&self, _package: &PackageName) -> Result<Option<PackageDetails>, ProbeError> {
        Ok(None)
    }
}

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn search(text: Option<&str>, installed: bool, orphans: bool) -> PackagesSearch {
    PackagesSearch {
        text: text.map(str::to_owned),
        installed,
        orphans,
        aur: false,
        read: Read::default(),
    }
}

fn names(rows: &[super::PackageRow]) -> Vec<&str> {
    rows.iter().map(|row| row.name.as_str()).collect()
}

fn args(text: &str) -> PackageArgs {
    PackageArgs {
        package: name(text),
        confirm: Confirm::default(),
    }
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("gimp")])));
    snapshot.insert(SyncPackages(BTreeSet::from([
        name("gimp"),
        name("inkscape"),
    ])));
    snapshot
}

#[test]
fn a_search_looks_at_every_package_unless_told_otherwise() {
    let all = found(&search(None, false, false), &Index).unwrap();
    assert_eq!(names(&all), ["gimp", "inkscape", "leftover"]);

    let text = found(&search(Some("INK"), false, false), &Index).unwrap();
    assert_eq!(names(&text), ["inkscape"]);

    let installed = found(&search(None, true, false), &Index).unwrap();
    assert_eq!(names(&installed), ["gimp", "leftover"]);

    let orphans = found(&search(None, false, true), &Index).unwrap();
    assert_eq!(names(&orphans), ["leftover"]);
}

#[test]
fn the_listing_marks_installed_packages_and_counts_them() {
    let rows = found(&search(None, false, false), &Index).unwrap();
    let listing = listing(&rows);

    assert_eq!(listing.summary.as_deref(), Some("3, 2 installed"));
    assert_eq!(listing.rows[1].cells[0], "inkscape");
}

#[test]
fn install_and_remove_plan_pacman_as_the_page_queues_it() {
    let install = change(&args("inkscape"), Toggle::On, &snapshot()).unwrap();
    assert!(matches!(
        &install.steps()[0].action,
        PlannedAction::Root(RootAction::InstallPackages { packages })
            if packages.iter().map(PackageName::as_str).eq(["inkscape"])
    ));

    let remove = change(&args("gimp"), Toggle::Off, &snapshot()).unwrap();
    assert!(matches!(
        &remove.steps()[0].action,
        PlannedAction::Root(RootAction::RemovePackages { packages: _ })
    ));
}

#[test]
fn a_change_the_system_has_already_is_nothing_to_do() {
    assert!(matches!(
        change(&args("gimp"), Toggle::On, &snapshot()),
        Err(Refused::NothingToDo(_))
    ));
    assert!(matches!(
        change(&args("inkscape"), Toggle::Off, &snapshot()),
        Err(Refused::NothingToDo(_))
    ));
}

#[test]
fn a_package_no_repository_offers_is_unknown_with_a_hint() {
    let refused = change(&args("inkscap"), Toggle::On, &snapshot());

    assert!(
        matches!(&refused, Err(Refused::Unknown(message)) if message.contains("inkscape")),
        "{refused:?}"
    );
}

#[test]
fn without_the_databases_the_change_is_unavailable() {
    assert!(matches!(
        change(&args("gimp"), Toggle::Off, &Snapshot::default()),
        Err(Refused::Unavailable(_) | Refused::NothingToDo(_))
    ));
}

#[test]
fn the_facts_name_every_field() {
    let info = PackageInfo {
        name: "gimp".to_owned(),
        description: "About gimp".to_owned(),
        version: "1.0-1".to_owned(),
        repository: Some("extra".to_owned()),
        size: 2048,
        url: None,
        licenses: vec!["GPL-3.0-or-later".to_owned()],
        groups: Vec::new(),
        depends: vec!["gegl".to_owned()],
        optional: Vec::new(),
        required_by: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        replaces: Vec::new(),
        installed: None,
    };

    let text = info_text(&info);
    for fact in ["extra", "2.0 KiB", "GPL-3.0-or-later", "depends on", "gegl"] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}
