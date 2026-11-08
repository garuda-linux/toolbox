mod filter;

use std::collections::BTreeSet;

use toolbox_catalog::ToggleCatalog;

use super::{AppOrder, Apps, Hit, arrange, icon_name};
use crate::domains::{InstalledPackages, Packages, SyncPackages};
use crate::features::search::Needle;
use crate::testing::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

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
url = "https://www.firefox.com/"

[[section.entry]]
id = "aerc"
title = "aerc"
description = "Asynchronous email client."
check = { package = "aerc" }
also = [{ package = "w3m" }]
"#;

const UTILITIES: &str = r#"
[[section]]
id = "utilities"
title = "Utilities"
icon = "applications-utilities"

[[section.entry]]
id = "kate"
title = "Kate"
description = "Advanced text editor with a browser for files."
check = { package = "kate" }

[[section.entry]]
id = "brave-bin"
title = "Brave"
description = "Web browser."
check = { package = "brave-bin" }
"#;

pub(super) fn apps() -> Apps {
    Apps::new(vec![
        ToggleCatalog::parse(INTERNET, "apps/internet.toml").unwrap(),
        ToggleCatalog::parse(UTILITIES, "apps/utilities.toml").unwrap(),
    ])
}

pub(super) fn ids(hits: &[Hit<'_>]) -> Vec<String> {
    hits.iter().map(|hit| hit.entry.id().to_owned()).collect()
}

pub(super) fn offering(list: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(InstalledPackages(BTreeSet::new()));
    snapshot.insert(SyncPackages(packages(list)));
    snapshot
}

#[test]
fn without_search_the_page_lists_one_category() {
    let apps = apps();
    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();

    assert_eq!(
        ids(&apps.filtered(Some("internet"), "", &snapshot, &changes)),
        ["firefox", "aerc"]
    );
    assert_eq!(
        ids(&apps.filtered(Some("utilities"), "  ", &snapshot, &changes)),
        ["kate", "brave-bin"]
    );
    assert_eq!(
        ids(&apps.filtered(Some("nothing"), "", &snapshot, &changes)),
        Vec::<String>::new()
    );
}

#[test]
fn a_search_looks_through_every_category_without_case() {
    let apps = apps();
    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();
    let hits = apps.filtered(None, "BROWSER", &snapshot, &changes);

    assert_eq!(ids(&hits), ["firefox", "kate", "brave-bin"]);
    assert_eq!(hits[1].category.title(), "Utilities");
    assert_eq!(
        ids(&apps.filtered(None, "w3m", &snapshot, &changes)),
        ["aerc"],
        "packages that come along count"
    );
    assert_eq!(
        ids(&apps.filtered(None, "brave-b", &snapshot, &changes)),
        ["brave-bin"]
    );
}

#[test]
fn apps_the_repositories_do_not_offer_are_hidden_and_not_counted() {
    let apps = apps();
    let snapshot = offering(&["firefox", "kate"]);
    let changes = ChangeSet::default();

    assert_eq!(
        ids(&apps.filtered(Some("internet"), "", &snapshot, &changes)),
        ["firefox"]
    );

    let counts: Vec<(&str, usize)> = apps
        .categories_matching("", &snapshot, &changes)
        .iter()
        .map(|category| (category.section.id(), category.count))
        .collect();

    assert_eq!(counts, [("internet", 1), ("utilities", 1)]);

    let none = offering(&["firefox"]);
    let left: Vec<&str> = apps
        .categories_matching("", &none, &changes)
        .iter()
        .map(|category| category.section.id())
        .collect();

    assert_eq!(left, ["internet"], "an empty category is hidden");
}

#[test]
fn a_click_queues_the_packages_and_a_second_takes_them_back() {
    let apps = apps();
    let snapshot = offering(&["aerc", "w3m"]);
    let mut changes = ChangeSet::default();

    assert!(apps.toggle("aerc", Toggle::On, &snapshot, &mut changes));
    assert_eq!(
        changes.wish::<Packages>(&package("aerc")),
        Some(&Toggle::On)
    );
    assert_eq!(changes.wish::<Packages>(&package("w3m")), Some(&Toggle::On));

    assert!(apps.toggle("aerc", Toggle::Off, &snapshot, &mut changes));
    assert!(changes.is_empty());
    assert!(!apps.toggle("unknown", Toggle::On, &snapshot, &mut changes));
}

#[test]
fn matching_takes_a_folded_needle() {
    let apps = apps();
    let firefox = apps.all().next().unwrap();

    assert!(firefox.matches(&Needle::new("mozilla")));
    assert!(firefox.matches(&Needle::new("Mozilla")));
    assert!(firefox.matches(&Needle::new(firefox.entry.title())));
}

#[test]
fn a_final_sigma_is_found_through_a_sigma() {
    const GREEK: &str = r#"
[[section]]
id = "greek"
title = "Greek"
icon = "applications-utilities"

[[section.entry]]
id = "nomos"
title = "ΝΟΜΟΣ"
description = "Law browser."
check = { package = "nomos" }
"#;

    let apps = Apps::new(vec![
        ToggleCatalog::parse(GREEK, "apps/greek.toml").unwrap(),
    ]);

    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();

    assert_eq!(
        ids(&apps.filtered(None, "νομοσ", &snapshot, &changes)),
        ["nomos"]
    );
    assert_eq!(
        ids(&apps.filtered(None, "νομο\u{3c2}", &snapshot, &changes)),
        ["nomos"],
        "the final sigma is one letter with the sigma"
    );
}

#[test]
fn icon_names_drop_the_build_ending() {
    assert_eq!(icon_name(&package("firefox")), "firefox");
    assert_eq!(icon_name(&package("brave-bin")), "brave");
    assert_eq!(icon_name(&package("neovim-git")), "neovim");
    assert_eq!(icon_name(&package("krita-appimage")), "krita");
    assert_eq!(icon_name(&package("virtualbox-meta")), "virtualbox");
    assert_eq!(icon_name(&package("git")), "git");
}

#[test]
fn apps_sort_by_name_without_case_also_across_categories() {
    let apps = apps();
    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();
    let mut category = apps.filtered(Some("internet"), "", &snapshot, &changes);
    let mut found = apps.filtered(None, "browser", &snapshot, &changes);

    arrange(&mut category, AppOrder::Name, &snapshot, &changes);
    arrange(&mut found, AppOrder::Name, &snapshot, &changes);

    assert_eq!(ids(&category), ["aerc", "firefox"]);
    assert_eq!(ids(&found), ["brave-bin", "firefox", "kate"]);
}

#[test]
fn selected_apps_come_first_installed_or_queued_and_a_queued_removal_does_not() {
    let apps = apps();
    let mut snapshot = Snapshot::default();

    snapshot.insert(InstalledPackages(packages(&["kate", "firefox"])));
    snapshot.insert(SyncPackages(packages(&[
        "aerc",
        "firefox",
        "kate",
        "brave-bin",
    ])));

    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("firefox"), Toggle::Off, &snapshot);
    changes.want::<Packages>(package("brave-bin"), Toggle::On, &snapshot);

    let mut found = apps.filtered(None, "e", &snapshot, &changes);

    arrange(&mut found, AppOrder::Selected, &snapshot, &changes);
    assert_eq!(ids(&found), ["brave-bin", "kate", "aerc", "firefox"]);
}

#[test]
fn a_title_search_ignores_descriptions_and_packages() {
    let apps = apps();
    let titled = |search: &str| -> Vec<String> {
        let needle = Needle::new(search);
        apps.all()
            .filter(|hit| hit.title_matches(&needle))
            .map(|hit| hit.entry.id().to_owned())
            .collect()
    };

    assert_eq!(titled("FIRE"), ["firefox"]);
    assert_eq!(titled("browser"), Vec::<String>::new());
    assert_eq!(titled("w3m"), Vec::<String>::new());
}
