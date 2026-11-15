mod gaps;
mod refusal;

use std::collections::BTreeSet;
use std::path::Path;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::domains::{InstalledPackages, Packages, SyncPackages};
use toolbox_core::testing::Queue as _;
use toolbox_core::{Snapshot, Toggle};
use toolbox_gui_kit::toasts::ToastKind;

use super::DiscoverPage;
use crate::catalogs::Catalogs;
use crate::command::{AppCommand, Command, DiscoverCommand};
use crate::testing::{
    PageFixture, assert_header, assert_rows_hold_their_parts, assert_text_beside_picture,
    body_font, picture_beside, rect_of,
};

/// The shipped apps, with Firefox and Floorp on offer and nothing
/// installed.
fn offered() -> PageFixture {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::new()));
    snapshot.insert(SyncPackages(BTreeSet::from([
        "firefox".parse().unwrap(),
        "floorp".parse().unwrap(),
    ])));

    PageFixture {
        catalogs: Catalogs::load(&dir),
        ..PageFixture::with_snapshot(snapshot)
    }
}

fn searching(title: &str) -> DiscoverPage {
    let mut page = DiscoverPage::default();
    page.reveal(title);
    page
}

fn searching_firefox() -> DiscoverPage {
    searching("Firefox")
}

#[test]
fn while_a_plan_runs_the_install_buttons_rest() {
    for busy in [true, false] {
        let fixture = PageFixture { busy, ..offered() };
        let mut harness = fixture.harness(searching_firefox());
        harness.run();

        let install = harness.get_by_role_and_label(Role::Button, "Install Firefox");
        assert_eq!(install.accesskit_node().is_disabled(), busy, "busy {busy}");
    }
}

#[test]
fn an_app_installs_with_a_button_under_its_category_heading() {
    let mut harness = offered().harness(searching_firefox());
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Firefox")
            .is_none()
    );
    harness
        .get_by_role_and_label(Role::Button, "Install Firefox")
        .click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Internet")
            .is_some()
    );

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(DiscoverCommand::Toggle {
            app: "firefox".to_owned(),
            wish: Toggle::On,
        })]
    );
}

#[test]
fn every_category_lists_under_its_own_heading() {
    let mut harness = offered().harness(DiscoverPage::default());
    harness.run();

    let headings: Vec<String> = harness
        .query_all_by_role(Role::Heading)
        .filter_map(|node| node.accesskit_node().label())
        .collect();

    assert!(
        headings.iter().any(|heading| heading == "Internet"),
        "{headings:?}"
    );
    assert!(
        harness.query_all_by_role(Role::List).next().is_some(),
        "the category's apps are a list"
    );
}

/// Firefox and Floorp match "Firefox" in Internet, Kate in Documents
/// does not.
#[test]
fn a_picked_category_without_a_match_offers_all_matches() {
    let mut fixture = offered();
    let mut snapshot = Snapshot::clone(&fixture.snapshot);
    snapshot.insert(SyncPackages(BTreeSet::from([
        "firefox".parse().unwrap(),
        "floorp".parse().unwrap(),
        "kate".parse().unwrap(),
    ])));
    fixture.snapshot = std::sync::Arc::new(snapshot);

    let mut harness = fixture.harness(searching_firefox());
    harness.state_mut().page.category = Some("documents".to_owned());
    harness.run();

    assert!(
        harness
            .query_by_label_contains("No app in Documents matches")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Show all 2 matches")
        .click();
    harness.run();

    assert_eq!(harness.state().page.category, None);
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Install Firefox")
            .is_some()
    );
}

#[test]
fn while_a_plan_runs_the_app_menu_stays_open_to_use() {
    let fixture = PageFixture {
        busy: true,
        ..offered()
    };

    let mut harness = fixture.harness(searching_firefox());
    harness.run();

    let menu = harness.get_by_role_and_label(Role::Button, "More for Firefox");
    assert!(!menu.accesskit_node().is_disabled());
}

#[test]
fn a_frame_with_the_same_inputs_builds_no_rows() {
    let mut harness = offered().harness(searching_firefox());
    harness.run();
    let built = harness.state().page.rows.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.rows.builds(), built);

    harness.state_mut().page.reveal("Floorp");
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 1);

    let state = harness.state_mut();
    let snapshot = std::sync::Arc::clone(&state.fixture.snapshot);
    let firefox = "firefox".parse().unwrap();
    state
        .fixture
        .changes
        .want::<Packages>(firefox, Toggle::On, &snapshot);
    state.fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 2);
}

#[test]
fn a_frame_with_the_same_search_builds_no_chips() {
    let mut harness = offered().harness(DiscoverPage::default());
    harness.run();
    let built = harness.state().page.chips.builds();

    harness.step();
    harness.state_mut().page.category = Some("internet".to_owned());
    harness.run();
    assert_eq!(harness.state().page.chips.builds(), built);

    harness.state_mut().page.reveal("Floorp");
    harness.run();
    assert_eq!(harness.state().page.chips.builds(), built + 1);

    harness.state_mut().fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.chips.builds(), built + 2);
}

#[test]
fn space_on_a_focused_app_asks_for_its_install() {
    let mut harness = offered().harness(searching_firefox());
    harness.run();
    harness
        .get_by_role_and_label(Role::ListItem, "Firefox")
        .focus();
    harness.run();

    harness.key_press(Key::Space);
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(DiscoverCommand::Toggle {
            app: "firefox".to_owned(),
            wish: Toggle::On,
        })]
    );
}

#[test]
fn copy_name_copies_the_app_and_says_so() {
    let mut harness = offered().harness(searching_firefox());
    harness.run();
    harness
        .get_by_role_and_label(Role::ListItem, "Firefox")
        .focus();
    harness.run();

    harness.key_press_modifiers(Modifiers::SHIFT, Key::F10);
    harness.run();
    harness
        .get_by_role_and_label(Role::MenuItem, "Copy name")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(AppCommand::Toast {
            kind: ToastKind::Info,
            text: "Copied “Firefox” to the clipboard.".to_owned(),
        })]
    );
}

#[test]
fn a_larger_desktop_font_makes_taller_rows_that_cut_no_text() {
    let height = |large: bool| {
        let mut harness = offered().harness(searching("browser"));
        if large {
            body_font(&harness.ctx, 20.0);
        }

        harness.run();
        assert_rows_hold_their_parts(&harness);
        harness
            .get_by_role_and_label(Role::ListItem, "Firefox")
            .rect()
            .height()
    };

    let (usual, large) = (height(false), height(true));
    assert!(
        large > usual,
        "{large} tall in a larger font, {usual} usually"
    );
}

#[test]
fn the_toolbar_lines_up_with_the_list() {
    let mut harness = offered().harness(DiscoverPage::default());
    harness.run();

    let search = rect_of(&harness, Role::TextInput, "Search apps");
    let sort = rect_of(&harness, Role::ComboBox, "Sort apps");
    let list = harness
        .get_all_by_role(Role::ListItem)
        .next()
        .unwrap()
        .rect();

    let texts = (
        "App Catalog",
        "Apps from the Arch wiki, sorted into categories. Changes wait in the queue until you apply them.",
    );

    assert_header(&harness, texts, None, Some(search));
    assert!(
        (sort.right() - list.right()).abs() < 0.5,
        "sort {sort:?}, list {list:?}"
    );
    assert!(
        (sort.center().y - search.center().y).abs() < 0.5,
        "not one toolbar row"
    );
}

#[test]
fn an_apps_title_and_description_start_one_picture_gap_right_of_its_picture() {
    let mut harness = offered().harness(searching_firefox());
    harness.run();

    let description = "Extensible browser from Mozilla based on Gecko with fast rendering.";
    let picture = picture_beside(&harness, "Firefox");
    assert_text_beside_picture(&harness, picture, ("Firefox", description), "Firefox");
}
