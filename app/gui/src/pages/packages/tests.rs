mod gaps;

use std::collections::BTreeSet;
use std::sync::Arc;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::{InstalledPackages, Packages as PackagesDomain, SyncPackages};
use toolbox_core::features::packages::list::{Catalog, Filter};
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, Installed, InstalledDetails, LocalNode, Origin, PackageDetails,
    PackageIndex,
};
use toolbox_core::{Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::{PackagesPage, row};
use crate::command::{Command, PackagesCommand};
use crate::packages::{Details, Packages, Read};
use crate::testing::{
    PageFixture, assert_header, assert_rows_hold_their_content, assert_text_beside_picture,
    body_font, measured, picture_beside, rect_of,
};

pub(super) fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn entry(text: &str, installed: Option<InstallReason>, origin: Origin) -> IndexEntry {
    IndexEntry {
        name: name(text),
        description: format!("About {text}"),
        version: "1.0-1".to_owned(),
        origin,
        size: 4 * 1024 * 1024,
        installed: installed.map(|reason| Installed {
            version: "1.0-1".to_owned(),
            reason,
        }),
    }
}

fn node(text: &str, reason: InstallReason, depends: &[&str]) -> LocalNode {
    LocalNode {
        name: name(text),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        reason,
    }
}

/// gimp with its dependency gegl, a leftover dependency, paru from
/// outside the repositories and inkscape on offer.
pub(super) fn catalog() -> Arc<Catalog> {
    let extra = || Origin::Repository("extra".to_owned());
    Arc::new(Catalog::new(PackageIndex {
        entries: vec![
            entry("gegl", Some(InstallReason::Dependency), extra()),
            entry("gimp", Some(InstallReason::Explicit), extra()),
            entry("inkscape", None, extra()),
            entry("leftover", Some(InstallReason::Dependency), extra()),
            entry("paru", Some(InstallReason::Explicit), Origin::Foreign),
        ],
        local: vec![
            node("gegl", InstallReason::Dependency, &[]),
            node("gimp", InstallReason::Explicit, &["gegl"]),
            node("leftover", InstallReason::Dependency, &[]),
            node("paru", InstallReason::Explicit, &[]),
        ],
    }))
}

pub(super) fn gimp_details() -> Details {
    Details {
        package: name("gimp"),
        info: Read::Known(Some(PackageDetails {
            name: name("gimp"),
            description: "About gimp".to_owned(),
            version: "1.0-1".to_owned(),
            origin: Origin::Repository("extra".to_owned()),
            size: 4 * 1024 * 1024,
            url: Some("https://www.gimp.org/".to_owned()),
            licenses: vec!["GPL-3.0-or-later".to_owned()],
            groups: Vec::new(),
            depends: vec!["gegl>=0.4".to_owned()],
            optional: Vec::new(),
            required_by: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            installed: Some(InstalledDetails {
                version: "1.0-1".to_owned(),
                reason: InstallReason::Explicit,
                date: Some(1_700_000_000),
                files: vec!["/usr/bin/gimp".to_owned()],
            }),
        })),
        history: Read::Known(Vec::new()),
    }
}

pub(super) fn fixture(details: Option<Details>) -> PageFixture {
    let mut snapshot = Snapshot::default();
    let installed = ["gegl", "gimp", "leftover", "paru"].map(name);
    snapshot.insert(InstalledPackages(BTreeSet::from(installed)));
    let offered = ["gegl", "gimp", "inkscape", "leftover"].map(name);
    snapshot.insert(SyncPackages(BTreeSet::from(offered)));

    PageFixture {
        packages: Packages::showing(Read::Known(catalog()), details),
        ..PageFixture::with_snapshot(snapshot)
    }
}

#[test]
fn the_start_view_lists_the_installed_packages_with_their_tags() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    for package in ["gegl", "gimp", "leftover", "paru"] {
        assert!(listed(&harness, package), "{package}");
    }

    assert!(!listed(&harness, "inkscape"));
    assert!(harness.query_by_label("Not from a repository").is_some());
    assert_eq!(harness.query_all_by_label("Dependency").count(), 2);
}

#[test]
fn the_orphans_chip_shows_what_nothing_needs() {
    let page = PackagesPage {
        filter: Filter::Orphans,
        ..PackagesPage::default()
    };

    let mut harness = fixture(None).harness(page);

    harness.run();

    assert!(listed(&harness, "leftover"));
    assert!(!listed(&harness, "gegl"));
}

#[test]
fn install_queues_an_offered_package() {
    let page = PackagesPage {
        filter: Filter::All,
        search: "inkscape".to_owned(),
        ..PackagesPage::default()
    };

    let mut harness = fixture(None).harness(page);

    harness.run();

    harness.get_by_label("Install inkscape").click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::Toggle {
            package: name("inkscape"),
            wish: Toggle::On,
        })]
    );
}

#[test]
fn remove_asks_first_and_names_what_goes_along() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    harness.get_by_label("Remove gimp").click();
    harness.run();
    assert!(harness.state().fixture.sent().is_empty(), "queued at once");
    assert!(
        harness
            .query_by_label("It takes along 1 dependency that nothing else needs:")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Remove")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::Toggle {
            package: name("gimp"),
            wish: Toggle::Off,
        })]
    );
}

#[test]
fn a_queued_wish_is_taken_back_from_its_row() {
    let mut fixture = fixture(None);
    let snapshot = Arc::clone(&fixture.snapshot);
    fixture
        .changes
        .want::<PackagesDomain>(name("paru"), Toggle::Off, &snapshot);
    let mut harness = fixture.harness(PackagesPage::default());
    harness.run();

    assert!(harness.query_by_label("Removes with Apply").is_some());
    harness.get_by_label("Take back paru").click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::Toggle {
            package: name("paru"),
            wish: Toggle::On,
        })]
    );
}

#[test]
fn space_on_a_row_opens_its_details_and_escape_goes_back() {
    let mut harness = fixture(Some(gimp_details())).harness(PackagesPage::default());
    harness.run();

    harness
        .get_by_role_and_label(Role::ListItem, "gimp")
        .focus();
    harness.run();
    harness.key_press(Key::Space);
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::Show(name("gimp")))]
    );
    assert!(harness.query_by_label("https://www.gimp.org/").is_some());
    assert!(harness.query_by_label("Installed explicitly").is_some());

    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert_eq!(harness.state().page.open, None);
    assert!(listed(&harness, "leftover"));
}

#[test]
fn an_unreadable_index_offers_to_read_again() {
    let fixture = PageFixture {
        packages: Packages::showing(Read::Failed, None),
        ..PageFixture::default()
    };

    let mut harness = fixture.harness(PackagesPage::default());

    harness.run();

    let banner = harness.query_all_by_label("Read again").last().unwrap();
    banner.click();
    harness.run();

    assert!(
        harness
            .state()
            .fixture
            .sent()
            .contains(&Command::from(PackagesCommand::Refresh))
    );
}

#[test]
fn the_toolbar_sits_under_the_header() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    let search = rect_of(&harness, Role::TextInput, "Search packages");
    let texts = (
        "Packages",
        "The packages of the repositories and every installed one. Installs and removals wait in the queue until you apply them.",
    );

    assert_header(&harness, texts, None, Some(search));
}

#[test]
fn a_larger_desktop_font_makes_taller_rows_that_cut_no_text() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    body_font(&harness.ctx, 20.0);
    harness.run();

    assert_rows_hold_their_content(&harness, measured(20.0, row::height));
}

pub(super) fn listed<S>(harness: &egui_kittest::Harness<'_, S>, package: &str) -> bool {
    harness
        .query_by_role_and_label(Role::ListItem, package)
        .is_some()
}

#[test]
fn a_rows_name_and_description_start_one_picture_gap_right_of_its_tile() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    let picture = picture_beside(&harness, "gimp");
    assert_text_beside_picture(&harness, picture, ("gimp", "About gimp"), "gimp");
}
