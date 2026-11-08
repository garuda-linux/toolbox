use toolbox_catalog::ToggleCatalog;

use super::{Tally, sections, unknown};
use crate::domains::{InstalledPackages, Packages, SyncPackages};
use crate::testing::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

const CATALOG: &str = r#"
[[section]]
id = "audio"
title = "Audio"
icon = "audio-volume-high"

[[section.entry]]
id = "pipewire-support"
title = "PipeWire"
description = "Sound."
check = { package = "pipewire-support" }

[[section.entry]]
id = "jack-support"
title = "JACK"
description = "Studio sound."
check = { package = "jack-support" }
requires = { all = ["pipewire-support"] }

[[section]]
id = "printing"
title = "Printing"
icon = "printer"

[[section.entry]]
id = "printing-support"
title = "Printing"
description = "Printers."
check = { package = "printing-support" }
"#;

fn catalog() -> ToggleCatalog {
    ToggleCatalog::parse(CATALOG, "components.toml").unwrap()
}

/// pipewire installed, everything but printing offered.
fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(&["pipewire-support"])));
    snapshot.insert(SyncPackages(packages(&[
        "pipewire-support",
        "jack-support",
    ])));
    snapshot
}

#[test]
fn entries_come_with_their_state_and_needs() {
    let catalog = catalog();
    let shown = sections(&catalog, &snapshot(), &ChangeSet::default(), &[]);

    let audio = shown.first().unwrap();
    assert_eq!(audio.section.id(), "audio");
    let ids: Vec<&str> = audio.entries.iter().map(|entry| entry.entry.id()).collect();
    assert_eq!(ids, ["pipewire-support", "jack-support"]);

    let pipewire = audio.entries.first().unwrap();
    assert!(pipewire.state.shown_on());
    assert!(pipewire.known());

    let jack = audio.entries.get(1).unwrap();
    assert!(!jack.state.shown_on());
    assert!(jack.changeable());
    let needs: Vec<&str> = jack.requires.iter().map(|entry| entry.title()).collect();
    assert_eq!(needs, ["PipeWire"]);
    assert!(!jack.requires_any);
}

#[test]
fn what_the_system_cannot_install_and_empty_sections_are_left_out() {
    let catalog = catalog();
    let shown = sections(&catalog, &snapshot(), &ChangeSet::default(), &[]);

    let ids: Vec<&str> = shown.iter().map(|section| section.section.id()).collect();
    assert_eq!(ids, ["audio"]);
}

#[test]
fn a_queued_wish_shows_and_counts_as_on() {
    let catalog = catalog();
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("jack-support"), Toggle::On, &snapshot);

    let shown = sections(&catalog, &snapshot, &changes, &[]);

    let jack = shown.first().unwrap().entries.get(1).unwrap();
    assert!(jack.state.shown_on());
    assert_eq!(jack.state.queued, Some(Toggle::On));
    assert_eq!(shown.first().unwrap().on(), 2);
}

#[test]
fn entries_without_a_read_state_are_counted() {
    let catalog = catalog();
    let mut snapshot = Snapshot::default();
    snapshot.insert(SyncPackages(packages(&[
        "pipewire-support",
        "jack-support",
    ])));

    let shown = sections(&catalog, &snapshot, &ChangeSet::default(), &[]);

    assert_eq!(unknown(&shown), 2);
}

#[test]
fn a_tally_counts_on_and_total_among_the_kept_entries() {
    let catalog = catalog();
    let shown = sections(&catalog, &snapshot(), &ChangeSet::default(), &[]);
    let audio = shown.first().unwrap();

    assert_eq!(audio.tally(|_| true), Tally { on: 1, total: 2 });
    assert_eq!(
        audio.tally(|entry| entry.entry.id() == "jack-support"),
        Tally { on: 0, total: 1 }
    );
    assert_eq!(audio.tally(|_| false), Tally { on: 0, total: 0 });
}

#[test]
fn tallies_add_up_to_the_page_count() {
    let tallies = [Tally { on: 1, total: 2 }, Tally { on: 0, total: 3 }];

    assert_eq!(
        tallies.into_iter().sum::<Tally>(),
        Tally { on: 1, total: 5 }
    );
    assert_eq!(std::iter::empty::<Tally>().sum::<Tally>(), Tally::default());
}

/// An installed package the protected catalog keeps shows on, with its
/// reason, and its switch rests.
#[test]
fn a_protected_installed_entry_is_kept() {
    use toolbox_state::packages::{ProtectedPackages, Protection};

    let catalog = catalog();
    let mut snapshot = snapshot();
    snapshot.insert(ProtectedPackages(std::collections::BTreeMap::from([(
        package("pipewire-support"),
        Protection::Desktop,
    )])));

    let shown = sections(&catalog, &snapshot, &ChangeSet::default(), &[]);
    let audio = shown.first().unwrap();
    let pipewire = audio.entries.first().unwrap();
    let jack = audio.entries.get(1).unwrap();

    assert_eq!(pipewire.state.kept, Some(Protection::Desktop));
    assert!(!pipewire.changeable());
    assert_eq!(jack.state.kept, None);
}
