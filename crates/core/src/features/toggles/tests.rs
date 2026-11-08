mod accounts;
mod checks;
mod gaming;
mod kernels;
mod setup;

use std::collections::BTreeMap;

use toolbox_catalog::{Check, ToggleCatalog};

use super::{EntryState, state, toggle};
use crate::domains::{
    CurrentUser, EnabledUnits, Groups, InstalledPackages, Packages, SystemUnits, UserConfigStates,
    UserConfigWish, UserConfigs,
};
use crate::testing::{membership, nina_in, package, packages, unit, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

const CATALOG: &str = r#"
[[section]]
id = "containers"
title = "Containers"
icon = "application-x-executable"

[[section.entry]]
id = "docker"
title = "Docker"
description = "The engine."
check = { package = "docker" }

[[section.entry]]
id = "docker-service"
title = "Docker service"
description = "Start it at boot."
check = { service = "docker.service" }
requires = { all = ["docker"] }

[[section.entry]]
id = "docker-group"
title = "Docker group"
description = "Use it without root."
check = { group = "docker" }
requires = { all = ["docker-service"] }
"#;

const EITHER: &str = r#"
[[section]]
id = "containers"
title = "Containers"
icon = "x"

[[section.entry]]
id = "docker"
title = "Docker"
description = "D"
check = { package = "docker" }

[[section.entry]]
id = "podman"
title = "Podman"
description = "P"
check = { package = "podman" }

[[section.entry]]
id = "distrobox"
title = "Distrobox"
description = "Works with either engine."
check = { package = "distrobox" }
requires = { any = ["docker", "podman"] }
"#;

const CONFIGS: &str = r#"
[[section]]
id = "desktop-and-apps"
title = "Desktop and apps"
icon = "x"

[[section.entry]]
id = "discord"
title = "Discord"
description = "D"
check = { user-json = { path = ".config/discord/settings.json", key = "SKIP_HOST_UPDATE" } }
restart-user-units = ["pipewire.service"]
"#;

fn catalog() -> ToggleCatalog {
    ToggleCatalog::parse(CATALOG, "test.toml").unwrap()
}

fn snapshot(installed: &[&str], enabled: &[&str], groups: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(InstalledPackages(packages(installed)));
    snapshot.insert(EnabledUnits(
        enabled.iter().map(|name| unit(name)).collect(),
    ));
    snapshot.insert(nina_in(groups));
    snapshot.insert(CurrentUser(user_name("nico")));
    snapshot
}

fn shown(id: &str, snapshot: &Snapshot, changes: &ChangeSet) -> EntryState {
    let catalog = catalog();

    state(&catalog, catalog.entry(id).unwrap(), snapshot, changes)
}

#[test]
fn an_installed_package_shows_on_and_nothing_is_queued() {
    let system = snapshot(&["docker"], &[], &[]);
    let docker = shown("docker", &system, &ChangeSet::default());

    assert_eq!(docker.system, Some(Toggle::On));
    assert_eq!(docker.queued, None);
    assert!(docker.shown_on() && docker.available);
}

#[test]
fn without_a_probe_the_state_is_unknown_and_shows_off() {
    let docker = shown("docker", &Snapshot::default(), &ChangeSet::default());

    assert_eq!(docker.system, None);
    assert!(!docker.shown_on());
}

#[test]
fn a_git_build_counts_as_installed_and_is_the_one_removed() {
    let system = snapshot(&["docker-git"], &[], &[]);
    let mut changes = ChangeSet::default();

    assert_eq!(shown("docker", &system, &changes).system, Some(Toggle::On));
    toggle(&catalog(), "docker", Toggle::Off, &system, &mut changes).unwrap();
    assert_eq!(
        changes.wish::<Packages>(&package("docker-git")),
        Some(&Toggle::Off)
    );
    assert_eq!(changes.wish::<Packages>(&package("docker")), None);
    assert_eq!(shown("docker", &system, &changes).queued, Some(Toggle::Off));
}

#[test]
fn a_click_queues_one_change_in_the_checks_domain() {
    let system = snapshot(&["docker"], &[], &[]);
    let mut changes = ChangeSet::default();

    toggle(
        &catalog(),
        "docker-service",
        Toggle::On,
        &system,
        &mut changes,
    )
    .unwrap();
    assert_eq!(
        changes.wish::<SystemUnits>(&unit("docker.service")),
        Some(&Toggle::On)
    );
    assert_eq!(changes.len(), 1);
}

#[test]
fn clicking_back_to_the_system_state_leaves_nothing_queued() {
    let system = snapshot(&["docker"], &[], &[]);
    let mut changes = ChangeSet::default();

    toggle(&catalog(), "docker", Toggle::Off, &system, &mut changes).unwrap();
    toggle(&catalog(), "docker", Toggle::On, &system, &mut changes).unwrap();

    assert!(changes.is_empty());
}

#[test]
fn an_entry_waits_for_what_it_requires() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    assert!(!shown("docker-service", &system, &changes).available);
    toggle(
        &catalog(),
        "docker-service",
        Toggle::On,
        &system,
        &mut changes,
    )
    .unwrap_err();
    assert!(changes.is_empty());

    toggle(&catalog(), "docker", Toggle::On, &system, &mut changes).unwrap();

    assert!(
        shown("docker-service", &system, &changes).available,
        "a queued requirement is enough"
    );
}

#[test]
fn turning_a_requirement_off_drops_what_was_queued_on_top_of_it() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    toggle(&catalog(), "docker", Toggle::On, &system, &mut changes).unwrap();
    toggle(
        &catalog(),
        "docker-service",
        Toggle::On,
        &system,
        &mut changes,
    )
    .unwrap();
    toggle(
        &catalog(),
        "docker-group",
        Toggle::On,
        &system,
        &mut changes,
    )
    .unwrap();
    assert_eq!(changes.len(), 3);

    toggle(&catalog(), "docker", Toggle::Off, &system, &mut changes).unwrap();

    assert!(changes.is_empty(), "the service and the group went too");
}

#[test]
fn turning_off_keeps_dependents_that_are_on_in_the_system() {
    let system = snapshot(&["docker"], &["docker.service"], &["docker"]);
    let mut changes = ChangeSet::default();

    toggle(&catalog(), "docker", Toggle::Off, &system, &mut changes).unwrap();

    assert_eq!(changes.len(), 1, "only the package removal is queued");
    assert_eq!(changes.wish::<Groups>(&membership("nico", "docker")), None);
}

#[test]
fn unknown_ids_change_nothing() {
    let mut changes = ChangeSet::default();

    toggle(
        &catalog(),
        "nope",
        Toggle::On,
        &Snapshot::default(),
        &mut changes,
    )
    .unwrap_err();
    assert!(changes.is_empty());
}

#[test]
fn any_of_its_requirements_is_enough() {
    let catalog = ToggleCatalog::parse(EITHER, "test.toml").unwrap();
    let system = snapshot(&["podman"], &[], &[]);
    let mut changes = ChangeSet::default();

    toggle(&catalog, "distrobox", Toggle::On, &system, &mut changes).unwrap();
    assert_eq!(changes.len(), 1);
}

#[test]
fn turning_one_engine_off_keeps_what_the_other_still_carries() {
    let catalog = ToggleCatalog::parse(EITHER, "test.toml").unwrap();
    let system = snapshot(&["podman"], &[], &[]);
    let mut changes = ChangeSet::default();

    toggle(&catalog, "docker", Toggle::On, &system, &mut changes).unwrap();
    toggle(&catalog, "distrobox", Toggle::On, &system, &mut changes).unwrap();
    toggle(&catalog, "docker", Toggle::Off, &system, &mut changes).unwrap();

    assert_eq!(
        changes.wish::<Packages>(&package("distrobox")),
        Some(&Toggle::On),
        "podman is still installed"
    );

    toggle(&catalog, "podman", Toggle::Off, &system, &mut changes).unwrap();

    assert_eq!(
        changes.wish::<Packages>(&package("distrobox")),
        None,
        "with neither engine left the queued distrobox goes"
    );
}

#[test]
fn clicking_a_queued_entry_takes_the_change_back_even_without_a_probe() {
    let mut changes = ChangeSet::default();

    toggle(
        &catalog(),
        "docker",
        Toggle::On,
        &Snapshot::default(),
        &mut changes,
    )
    .unwrap();
    toggle(
        &catalog(),
        "docker",
        Toggle::Off,
        &Snapshot::default(),
        &mut changes,
    )
    .unwrap();

    assert!(changes.is_empty());
}

#[test]
fn a_user_config_entry_shows_its_probed_state_and_queues_with_its_restart() {
    let catalog = ToggleCatalog::parse(CONFIGS, "settings.toml").unwrap();
    let entry = catalog.entry("discord").unwrap();
    let Check::UserConfig(edit) = entry.check() else {
        panic!("not a user config");
    };

    let mut system = Snapshot::default();
    system.insert(UserConfigStates(BTreeMap::from([(edit.clone(), false)])));

    let mut changes = ChangeSet::default();

    assert_eq!(
        state(&catalog, entry, &system, &changes).system,
        Some(Toggle::Off)
    );
    toggle(&catalog, "discord", Toggle::On, &system, &mut changes).unwrap();

    let wish = UserConfigWish::new(Toggle::On, [unit("pipewire.service")]);

    assert_eq!(changes.wish::<UserConfigs>(edit), Some(&wish));
    assert!(state(&catalog, entry, &system, &changes).shown_on());

    toggle(&catalog, "discord", Toggle::Off, &system, &mut changes).unwrap();

    assert!(changes.is_empty());
}
