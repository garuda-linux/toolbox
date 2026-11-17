use std::collections::BTreeSet;

use toolbox_plan::PackageName;
use toolbox_state::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, Origin, PackageIndex,
};

use super::{Catalog, Counts, Filter, RowState, row_state};
use crate::domains::{InstalledPackages, Packages};
use crate::features::search::ranked::Rank;
use crate::{ChangeSet, Snapshot, Toggle};

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn offered(text: &str, description: &str) -> IndexEntry {
    IndexEntry {
        name: name(text),
        description: toolbox_text::plain(description),
        version: "1.0-1".to_owned(),
        origin: Origin::Repository("extra".to_owned()),
        size: 1024,
        installed: None,
    }
}

fn installed(text: &str, reason: InstallReason) -> IndexEntry {
    IndexEntry {
        installed: Some(Installed {
            version: "1.0-1".to_owned(),
            reason,
        }),
        ..offered(text, &format!("About {text}"))
    }
}

fn node(text: &str, reason: InstallReason, depends: &[&str]) -> LocalNode {
    LocalNode {
        name: name(text),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        optional: Vec::new(),
        reason,
    }
}

/// gimp with its dependency gegl, a leftover dependency, a foreign
/// package and two packages only offered.
fn catalog() -> Catalog {
    let foreign = IndexEntry {
        origin: Origin::Foreign,
        ..installed("paru", InstallReason::Explicit)
    };

    Catalog::new(PackageIndex {
        entries: vec![
            offered("firefox", "Standalone web browser"),
            installed("gegl", InstallReason::Dependency),
            installed("gimp", InstallReason::Explicit),
            offered("inkscape", "Vector graphics editor"),
            installed("leftover", InstallReason::Dependency),
            foreign,
        ],
        local: vec![
            node("gegl", InstallReason::Dependency, &[]),
            node("gimp", InstallReason::Explicit, &["gegl"]),
            node("leftover", InstallReason::Dependency, &[]),
            node("paru", InstallReason::Explicit, &[]),
        ],
    })
}

fn names_of(catalog: &Catalog, rows: &[usize]) -> Vec<String> {
    rows.iter()
        .map(|&row| catalog.entries()[row].name.as_str().to_owned())
        .collect()
}

fn no_updates() -> BTreeSet<PackageName> {
    BTreeSet::new()
}

#[test]
fn installed_lists_every_installed_package_foreign_ones_too() {
    let catalog = catalog();
    let view = catalog.view(Filter::Installed, "", &no_updates());

    assert_eq!(
        names_of(&catalog, &view.rows),
        ["gegl", "gimp", "leftover", "paru"]
    );
}

#[test]
fn all_lists_every_offered_and_every_installed_package() {
    let catalog = catalog();
    let view = catalog.view(Filter::All, "", &no_updates());

    assert_eq!(view.rows.len(), 6);
}

#[test]
fn orphans_are_the_dependencies_nothing_needs() {
    let catalog = catalog();
    let view = catalog.view(Filter::Orphans, "", &no_updates());

    assert_eq!(names_of(&catalog, &view.rows), ["leftover"]);
    assert!(catalog.is_orphan(&name("leftover")));
    assert!(!catalog.is_orphan(&name("gegl")));
}

#[test]
fn updates_are_the_installed_packages_with_a_pending_update() {
    let catalog = catalog();
    let updates = BTreeSet::from([name("gimp"), name("not-installed")]);
    let view = catalog.view(Filter::Updates, "", &updates);

    assert_eq!(names_of(&catalog, &view.rows), ["gimp"]);
}

#[test]
fn the_search_looks_in_names_and_descriptions_whatever_the_case() {
    let catalog = catalog();

    let by_description = catalog.view(Filter::All, "VECTOR", &no_updates());
    assert_eq!(names_of(&catalog, &by_description.rows), ["inkscape"]);

    let by_name = catalog.view(Filter::All, " gim ", &no_updates());
    assert_eq!(names_of(&catalog, &by_name.rows), ["gimp"]);
}

#[test]
fn the_counts_follow_the_search() {
    let catalog = catalog();

    assert_eq!(
        catalog
            .view(Filter::Installed, "", &BTreeSet::from([name("gimp")]))
            .counts,
        Counts {
            installed: 4,
            updates: 1,
            orphans: 1,
            optional: 0,
            all: 6,
        }
    );
    assert_eq!(
        catalog.view(Filter::Installed, "ge", &no_updates()).counts,
        Counts {
            installed: 1,
            updates: 0,
            orphans: 0,
            optional: 0,
            all: 1,
        }
    );
}

#[test]
fn an_entry_is_found_by_name() {
    let catalog = catalog();

    assert_eq!(
        catalog.entry(&name("inkscape")).map(|entry| entry.size),
        Some(1024)
    );
    assert!(catalog.entry(&name("nothing")).is_none());
}

#[test]
fn a_row_shows_the_installed_state_until_a_wish_is_queued() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("gimp")])));
    let mut changes = ChangeSet::default();

    assert_eq!(
        row_state(&name("gimp"), &snapshot, &changes),
        RowState {
            installed: true,
            queued: None,
        }
    );

    changes.want::<Packages>(name("gimp"), Toggle::Off, &snapshot);
    changes.want::<Packages>(name("inkscape"), Toggle::On, &snapshot);

    assert_eq!(
        row_state(&name("gimp"), &snapshot, &changes),
        RowState {
            installed: true,
            queued: Some(Toggle::Off),
        }
    );
    assert!(!row_state(&name("gimp"), &snapshot, &changes).shown_installed());
    assert!(row_state(&name("inkscape"), &snapshot, &changes).shown_installed());
}

#[test]
fn every_filter_has_a_code() {
    assert_eq!(
        [
            Filter::Installed,
            Filter::Updates,
            Filter::Orphans,
            Filter::Optional,
            Filter::All,
            Filter::Aur
        ]
        .map(Filter::as_str),
        ["installed", "updates", "orphans", "optional", "all", "aur"]
    );
}

#[test]
fn the_aur_chip_lists_no_repository_package() {
    let catalog = catalog();
    let view = catalog.view(Filter::Aur, "", &no_updates());

    assert_eq!(view.rows, Vec::<usize>::new());
    assert_eq!(view.counts.all, 6);
}

#[test]
fn a_dependency_only_listed_as_optional_has_its_own_chip() {
    let catalog = Catalog::new(PackageIndex {
        entries: vec![
            installed("atomicparsley", InstallReason::Dependency),
            installed("leftover", InstallReason::Dependency),
            installed("yt-dlp", InstallReason::Explicit),
        ],
        local: vec![
            node("atomicparsley", InstallReason::Dependency, &[]),
            node("leftover", InstallReason::Dependency, &[]),
            LocalNode {
                optional: vec!["atomicparsley".to_owned()],
                ..node("yt-dlp", InstallReason::Explicit, &[])
            },
        ],
    });

    let optional = catalog.view(Filter::Optional, "", &no_updates());
    let orphans = catalog.view(Filter::Orphans, "", &no_updates());

    assert_eq!(names_of(&catalog, &optional.rows), ["atomicparsley"]);
    assert_eq!(names_of(&catalog, &orphans.rows), ["leftover"]);
    assert_eq!((optional.counts.optional, optional.counts.orphans), (1, 1));
    assert_eq!(
        catalog.optional_for(&name("atomicparsley")),
        [name("yt-dlp")]
    );
    assert_eq!(
        catalog.optional_for(&name("leftover")),
        Vec::<PackageName>::new()
    );
}

#[test]
fn ranked_packages_go_by_name_then_by_description() {
    let catalog = catalog();
    let needle = crate::features::search::Needle::new("gimp");
    let ranked: Vec<(Rank, &str)> = catalog
        .ranked(&needle)
        .map(|(rank, entry)| (rank, entry.name.as_str()))
        .collect();

    assert_eq!(ranked, [(Rank::Exact, "gimp")]);

    let about = crate::features::search::Needle::new("about gegl");
    let by_text: Vec<Rank> = catalog.ranked(&about).map(|(rank, _)| rank).collect();
    assert_eq!(by_text, [Rank::InText]);
}
