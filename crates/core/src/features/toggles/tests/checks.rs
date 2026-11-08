use std::collections::BTreeSet;

use toolbox_catalog::ToggleCatalog;
use toolbox_plan::UnitName;

use super::super::{state, toggle};
use crate::domains::{Accounts, CurrentUser, EnabledUserUnits, Groups, Packages, UserUnits};
use crate::testing::{group, membership, package, unit, user, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

const CHECKS: &str = r#"
[[section]]
id = "background"
title = "Background"
icon = "x"

[[section.entry]]
id = "timer-user"
title = "User timer"
description = "U"
check = { user-service = "backup.timer" }

[[section.entry]]
id = "primary-group"
title = "Primary group"
description = "G"
check = { group = "nico" }

[[section.entry]]
id = "shell"
title = "Shell"
description = "S"
check = { package = "fish" }
conflicts = ["primary-group"]
"#;

fn checks() -> ToggleCatalog {
    ToggleCatalog::parse(CHECKS, "test.toml").unwrap()
}

fn timer() -> UnitName {
    unit("backup.timer")
}

fn nina_primary() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts {
        users: vec![user("nico", 1000, 1000)],
        groups: vec![group("nico", 1000, &["nico"])],
    });
    snapshot.insert(CurrentUser(user_name("nico")));
    snapshot
}

#[test]
fn a_user_service_queues_in_the_user_units() {
    let catalog = checks();
    let mut system = Snapshot::default();
    system.insert(EnabledUserUnits::default());
    let mut changes = ChangeSet::default();

    toggle(&catalog, "timer-user", Toggle::On, &system, &mut changes).unwrap();

    assert_eq!(changes.wish::<UserUnits>(&timer()), Some(&Toggle::On));
    assert_eq!(
        state(
            &catalog,
            catalog.entry("timer-user").unwrap(),
            &system,
            &changes
        )
        .queued,
        Some(Toggle::On)
    );

    toggle(&catalog, "timer-user", Toggle::Off, &system, &mut changes).unwrap();

    assert!(changes.is_empty());
}

#[test]
fn an_enabled_user_service_shows_on() {
    let catalog = checks();
    let mut system = Snapshot::default();
    system.insert(EnabledUserUnits(BTreeSet::from([timer()])));

    assert_eq!(
        state(
            &catalog,
            catalog.entry("timer-user").unwrap(),
            &system,
            &ChangeSet::default()
        )
        .system,
        Some(Toggle::On)
    );
}

#[test]
fn a_group_shows_its_membership() {
    let catalog = checks();
    let member = state(
        &catalog,
        catalog.entry("primary-group").unwrap(),
        &nina_primary(),
        &ChangeSet::default(),
    );

    assert_eq!(member.system, Some(Toggle::On));

    let unknown = state(
        &catalog,
        catalog.entry("primary-group").unwrap(),
        &Snapshot::default(),
        &ChangeSet::default(),
    );

    assert_eq!(unknown.system, None);
}

#[test]
fn wanting_a_group_without_a_known_user_queues_nothing() {
    let mut changes = ChangeSet::default();

    toggle(
        &checks(),
        "primary-group",
        Toggle::On,
        &Snapshot::default(),
        &mut changes,
    )
    .unwrap();

    assert!(changes.is_empty());
}

#[test]
fn a_rival_that_refuses_stays_on() {
    let catalog = checks();
    let system = nina_primary();
    let mut changes = ChangeSet::default();

    toggle(&catalog, "shell", Toggle::On, &system, &mut changes).unwrap();

    assert_eq!(
        changes.wish::<Packages>(&package("fish")),
        Some(&Toggle::On)
    );
    assert_eq!(
        changes.wish::<Groups>(&membership("nico", "nico")),
        None,
        "the refused removal is not queued"
    );
}
