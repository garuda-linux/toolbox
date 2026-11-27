use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_catalog::ToggleCatalog;
use toolbox_core::features::discover::Apps;

use super::super::DiscoverPage;
use super::details::{Harnessed, open_id};
use super::{offered, offering, searching_firefox};
use crate::catalogs::{AppCatalogs, Catalogs};
use crate::pages::under_palette::{UnderPalette, escape_under_palette};
use crate::testing::PageFixture;

const TWINS: &str = r#"
[[section]]
id = "internet"
title = "Internet"
icon = "applications-internet"

[[section.entry]]
id = "twin-a"
title = "Twin"
description = "The first of two apps of one title."
check = { package = "twin-a" }

[[section.entry]]
id = "twin-b"
title = "Twin"
description = "The second of two apps of one title."
check = { package = "twin-b" }
"#;

/// Two apps of one title, both on offer.
fn twins() -> PageFixture {
    let apps = Apps::new(vec![
        ToggleCatalog::parse(TWINS, "apps/twins.toml").unwrap(),
    ]);

    let catalogs = Catalogs {
        apps: AppCatalogs {
            apps,
            problem: None,
        },
        ..Catalogs::default()
    };

    offering(catalogs, &["twin-a", "twin-b"])
}

/// Which of the rows titled `title` holds the focus, in list order.
fn focused_of(harness: &Harnessed, title: &str) -> Option<usize> {
    harness
        .query_all_by_role_and_label(Role::ListItem, title)
        .position(|row| row.is_focused())
}

#[test]
fn escape_focuses_the_closed_apps_row_and_not_another_of_its_title() {
    let mut harness = twins().harness(DiscoverPage::default());
    harness.run();

    harness
        .query_all_by_role_and_label(Role::ListItem, "Twin")
        .nth(1)
        .expect("two rows titled Twin")
        .click();
    harness.run();
    let opened = open_id(&harness).map(str::to_owned);
    assert!(opened.is_some(), "the second row opens its details");

    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert!(harness.state().page.open.is_none());
    assert_eq!(focused_of(&harness, "Twin"), Some(1), "{opened:?}");
}

#[test]
fn escape_closes_the_palette_over_the_details_and_the_details_stay() {
    let page = UnderPalette {
        page: searching_firefox(),
        ..UnderPalette::default()
    };

    let mut harness = offered().harness(page);
    harness.run();
    harness
        .get_by_role_and_label(Role::ListItem, "Firefox")
        .click();
    harness.run();

    escape_under_palette(&mut harness);
    let open = harness.state().page.page.open.as_ref();
    assert_eq!(open.map(|open| open.id.as_str()), Some("firefox"));
}
