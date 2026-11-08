use std::collections::BTreeSet;

use toolbox_catalog::ToggleCatalog;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::discover::{AppOrder, Apps};
use toolbox_core::features::toggles::is_offered;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::{Query, Shown, category, rows};

const INTERNET: &str = r#"
[[section]]
id = "internet"
title = "Internet"
icon = "applications-internet"

[[section.entry]]
id = "firefox"
title = "Firefox"
description = "Extensible browser from Mozilla."
check = { package = "firefox" }

[[section.entry]]
id = "aerc"
title = "aerc"
description = "Asynchronous email client."
check = { package = "aerc" }
"#;

const SCIENCE: &str = r#"
[[section]]
id = "science"
title = "Science"
icon = "applications-science"

[[section.entry]]
id = "stellarium"
title = "Stellarium"
description = "Planetarium with a browser for the sky."
check = { package = "stellarium" }
"#;

fn apps() -> Apps {
    Apps::new(vec![
        ToggleCatalog::parse(INTERNET, "apps/internet.toml").unwrap(),
        ToggleCatalog::parse(SCIENCE, "apps/science.toml").unwrap(),
    ])
}

fn system(installed: &[&str], offered: &[&str]) -> Snapshot {
    let names = |list: &[&str]| -> BTreeSet<PackageName> {
        list.iter().map(|name| name.parse().unwrap()).collect()
    };

    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(names(installed)));
    snapshot.insert(SyncPackages(names(offered)));
    snapshot
}

fn ids<'apps>(shown: &[Shown<'apps>]) -> Vec<&'apps str> {
    shown.iter().map(|row| row.hit.entry.id()).collect()
}

#[test]
fn all_categories_show_until_one_is_picked() {
    let apps = apps();
    let snapshot = Snapshot::default();
    let categories = apps.categories_matching("", &snapshot, &ChangeSet::default());

    assert_eq!(category(&categories, None), None);
    assert_eq!(category(&categories, Some("science")), Some("science"));
}

#[test]
fn a_picked_category_without_apps_gives_way_to_all() {
    let apps = apps();
    let snapshot = system(&[], &["firefox"]);
    let categories = apps.categories_matching("", &snapshot, &ChangeSet::default());

    assert_eq!(category(&categories, Some("science")), None);
    assert_eq!(category(&[], Some("science")), None);
}

#[test]
fn rows_carry_their_state_in_the_chosen_order() {
    let apps = apps();
    let snapshot = system(&["firefox"], &["firefox", "aerc", "stellarium"]);
    let changes = ChangeSet::default();

    let by_name = rows(
        &apps,
        query(Some("internet"), "", AppOrder::Name),
        &snapshot,
        &changes,
    );

    assert_eq!(ids(&by_name), ["aerc", "firefox"]);

    assert!(by_name[1].state.shown_on());
    assert!(!by_name[0].state.shown_on());

    let selected = rows(
        &apps,
        query(Some("internet"), "", AppOrder::Selected),
        &snapshot,
        &changes,
    );

    assert_eq!(ids(&selected), ["firefox", "aerc"]);
}

#[test]
fn a_search_keeps_to_the_chosen_category() {
    let apps = apps();
    let snapshot = system(&[], &["firefox", "aerc", "stellarium"]);
    let changes = ChangeSet::default();

    let found = rows(
        &apps,
        query(Some("internet"), "browser", AppOrder::Name),
        &snapshot,
        &changes,
    );

    assert_eq!(ids(&found), ["firefox"]);
}

#[test]
fn a_search_in_all_looks_through_every_category() {
    let apps = apps();
    let snapshot = system(&[], &["firefox", "aerc", "stellarium"]);
    let changes = ChangeSet::default();

    let found = rows(
        &apps,
        query(None, "browser", AppOrder::Name),
        &snapshot,
        &changes,
    );

    assert_eq!(ids(&found), ["firefox", "stellarium"]);
}

#[test]
fn a_queued_app_shows_its_wish() {
    let apps = apps();
    let snapshot = system(&[], &["firefox", "aerc"]);
    let mut changes = ChangeSet::default();
    assert!(apps.toggle("aerc", Toggle::On, &snapshot, &mut changes));

    let shown = rows(
        &apps,
        query(Some("internet"), "", AppOrder::Name),
        &snapshot,
        &changes,
    );

    assert_eq!(shown[0].state.queued, Some(Toggle::On));
    assert!(is_offered(shown[0].hit.entry, &snapshot, &changes));
}

fn query<'text>(category: Option<&'text str>, search: &'text str, order: AppOrder) -> Query<'text> {
    Query {
        category,
        search,
        order,
    }
}
