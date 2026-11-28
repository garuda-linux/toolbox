use std::collections::BTreeSet;

use toolbox_cli::args::{Confirm, PackageArgs, PackagesOnly, PackagesSearch, Read};
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, Origin, PackageDetails, PackageIndex,
};
use toolbox_core::{Snapshot, Toggle};
use toolbox_plan::{PackageName, PlannedAction, RootAction};
use toolbox_state::packages::PackageGraph;
use toolbox_system::{PackageIndexSource, ProbeError};

use super::{PackageInfo, change, found, info_text, listing};
use crate::change::Refused;

/// gimp installed with its leftover dependency, inkscape on offer.
struct Index;

impl PackageIndexSource for Index {
    fn index(&self) -> Result<PackageIndex, ProbeError> {
        let entry = |text: &str, installed: Option<InstallReason>| IndexEntry {
            name: name(text),
            description: toolbox_text::plain(&format!("About {text}")),
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
                entry("babl", Some(InstallReason::Dependency)),
                entry("gimp", Some(InstallReason::Explicit)),
                entry("inkscape", None),
                entry("leftover", Some(InstallReason::Dependency)),
            ],
            local: vec![
                LocalNode {
                    name: name("gimp"),
                    depends: Vec::new(),
                    provides: Vec::new(),
                    optional: vec!["babl".to_owned()],
                    reason: InstallReason::Explicit,
                },
                LocalNode {
                    name: name("babl"),
                    depends: Vec::new(),
                    provides: Vec::new(),
                    optional: Vec::new(),
                    reason: InstallReason::Dependency,
                },
                LocalNode {
                    name: name("leftover"),
                    depends: Vec::new(),
                    provides: Vec::new(),
                    optional: Vec::new(),
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
        only: PackagesOnly {
            installed,
            orphans,
            optional: false,
        },
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
    snapshot.insert(PackageGraph::default());
    snapshot
}

#[test]
fn a_search_looks_at_every_package_unless_told_otherwise() {
    let all = found(&search(None, false, false), &Index).unwrap();
    assert_eq!(names(&all), ["babl", "gimp", "inkscape", "leftover"]);

    let text = found(&search(Some("INK"), false, false), &Index).unwrap();
    assert_eq!(names(&text), ["inkscape"]);

    let installed = found(&search(None, true, false), &Index).unwrap();
    assert_eq!(names(&installed), ["babl", "gimp", "leftover"]);

    let orphans = found(&search(None, false, true), &Index).unwrap();
    assert_eq!(names(&orphans), ["leftover"]);

    let only_optional = PackagesSearch {
        only: PackagesOnly {
            optional: true,
            ..PackagesOnly::default()
        },
        ..search(None, false, false)
    };

    let optional = found(&only_optional, &Index).unwrap();
    assert_eq!(names(&optional), ["babl"], "babl is optional for gimp");
}

#[test]
fn the_listing_marks_installed_packages_and_counts_them() {
    let rows = found(&search(None, false, false), &Index).unwrap();
    let listing = listing(&rows);

    assert_eq!(listing.summary.as_deref(), Some("4, 3 installed"));
    assert_eq!(listing.rows[2].cells[0], "inkscape");
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

/// The package name is a plain string in JSON.
#[test]
fn a_package_row_as_json_keeps_its_name_as_a_string() {
    use super::PackageRow;

    let row = PackageRow {
        name: name("gimp"),
        description: toolbox_text::plain("Image editor"),
        version: "2.10-1".to_owned(),
        repository: Some("extra".to_owned()),
        size: 2048,
        installed: None,
    };

    assert_eq!(
        serde_json::to_string(&row).unwrap(),
        r#"{"id":"gimp","description":"Image editor","version":"2.10-1","repository":"extra","size":2048,"installed":null}"#
    );
}

#[test]
fn the_facts_name_every_field() {
    let info = PackageInfo {
        name: PackageName::new("gimp").unwrap(),
        description: toolbox_text::plain("About gimp"),
        version: "1.0-1".to_owned(),
        repository: Some("extra".to_owned()),
        size: 2048,
        url: None,
        licenses: vec![toolbox_text::plain("GPL-3.0-or-later")],
        groups: Vec::new(),
        depends: vec!["gegl".to_owned()],
        optional: Vec::new(),
        required_by: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        replaces: Vec::new(),
        installed: None,
    };

    let text = info_text(crate::out::paint(crate::out::Settings::default()), &info);
    for fact in ["extra", "2.0 KiB", "GPL-3.0-or-later", "depends on", "gegl"] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}

/// A change the databases refuse under `--json` is one JSON Lines error on
/// standard output, with the unavailable exit code.
#[test]
fn a_refused_package_change_under_json_is_one_error_line() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let planned = change(&args("gimp"), Toggle::On, &Snapshot::default());
    let confirm = Confirm {
        yes: true,
        json: true,
        ..Confirm::default()
    };

    let mut out = Vec::new();
    let exit = crate::act::run(
        planned,
        confirm,
        None,
        &runtime,
        &mut crate::out::Out::new(&mut out, crate::out::Settings::default()),
    );

    assert_eq!(exit, toolbox_cli::Exit::Unavailable);
    assert_eq!(
        String::from_utf8(out).unwrap(),
        concat!(
            r#"{"schema":1,"event":"error","message":"not available: pacman's databases could not be read"}"#,
            "\n"
        )
    );
}
