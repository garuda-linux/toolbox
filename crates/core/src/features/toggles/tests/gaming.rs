use toolbox_catalog::ToggleCatalog;

use super::snapshot;
use crate::domains::{Packages, SyncPackages};
use crate::features::toggles::{is_offered, state, toggle};
use crate::testing::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

const GAMING: &str = r#"
[[section]]
id = "tools"
title = "Tools"
icon = "x"

[[section.entry]]
id = "gamemode"
title = "Gamemode"
description = "G"
check = { package = "gamemode" }
also = [{ package = "lib32-gamemode" }]

[[section.entry]]
id = "steam"
title = "Steam"
description = "S"
check = { package = "steam" }

[[section.entry]]
id = "group-games"
title = "Games group"
description = "A group needs no repository."
check = { group = "games" }
"#;

fn catalog() -> ToggleCatalog {
    ToggleCatalog::parse(GAMING, "test.toml").unwrap()
}

fn offering(mut snapshot: Snapshot, offered: &[&str]) -> Snapshot {
    snapshot.insert(SyncPackages(packages(offered)));
    snapshot
}

fn offered(id: &str, snapshot: &Snapshot, changes: &ChangeSet) -> bool {
    is_offered(catalog().entry(id).unwrap(), snapshot, changes)
}

#[test]
fn turning_on_installs_the_also_packages_too() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    toggle(&catalog(), "gamemode", Toggle::On, &system, &mut changes).unwrap();
    assert_eq!(
        changes.wish::<Packages>(&package("gamemode")),
        Some(&Toggle::On)
    );
    assert_eq!(
        changes.wish::<Packages>(&package("lib32-gamemode")),
        Some(&Toggle::On)
    );
    assert_eq!(changes.len(), 2);
}

#[test]
fn turning_off_removes_only_the_also_packages_that_are_installed() {
    let both = snapshot(&["gamemode", "lib32-gamemode"], &[], &[]);
    let main_only = snapshot(&["gamemode"], &[], &[]);
    let mut both_off = ChangeSet::default();
    let mut main_off = ChangeSet::default();

    toggle(&catalog(), "gamemode", Toggle::Off, &both, &mut both_off).unwrap();
    assert_eq!(both_off.len(), 2);
    toggle(
        &catalog(),
        "gamemode",
        Toggle::Off,
        &main_only,
        &mut main_off,
    )
    .unwrap();
    assert_eq!(
        main_off.wish::<Packages>(&package("gamemode")),
        Some(&Toggle::Off)
    );
    assert_eq!(main_off.len(), 1);
}

#[test]
fn the_state_follows_the_main_package_and_a_second_click_takes_all_back() {
    let only_library = snapshot(&["lib32-gamemode"], &[], &[]);
    let mut changes = ChangeSet::default();
    let gamemode = catalog();
    let entry = gamemode.entry("gamemode").unwrap();

    assert_eq!(
        state(&gamemode, entry, &only_library, &changes).system,
        Some(Toggle::Off)
    );
    toggle(
        &gamemode,
        "gamemode",
        Toggle::On,
        &only_library,
        &mut changes,
    )
    .unwrap();
    assert_eq!(changes.len(), 1, "the installed library is no change");
    toggle(
        &gamemode,
        "gamemode",
        Toggle::Off,
        &only_library,
        &mut changes,
    )
    .unwrap();
    assert!(changes.is_empty());
}

#[test]
fn while_the_repositories_are_unknown_every_entry_is_offered() {
    let system = snapshot(&[], &[], &[]);
    let changes = ChangeSet::default();

    assert!(offered("steam", &system, &changes));
    assert!(offered("steam", &Snapshot::default(), &changes));
}

#[test]
fn a_package_neither_installed_nor_in_a_repository_is_not_offered() {
    let changes = ChangeSet::default();
    let none = offering(snapshot(&[], &[], &[]), &["gamemode"]);
    let installed = offering(snapshot(&["steam"], &[], &[]), &[]);
    let git_build = offering(snapshot(&["steam-git"], &[], &[]), &[]);

    assert!(!offered("steam", &none, &changes));
    assert!(offered("gamemode", &none, &changes));
    assert!(offered("steam", &installed, &changes));
    assert!(offered("steam", &git_build, &changes));
}

#[test]
fn a_queued_change_keeps_the_entry_and_other_checks_are_always_offered() {
    let unknown = snapshot(&[], &[], &[]);
    let none = offering(snapshot(&[], &[], &[]), &[]);
    let mut changes = ChangeSet::default();

    toggle(&catalog(), "steam", Toggle::On, &unknown, &mut changes).unwrap();
    assert!(offered("steam", &none, &changes));
    assert!(offered("group-games", &none, &ChangeSet::default()));
}
