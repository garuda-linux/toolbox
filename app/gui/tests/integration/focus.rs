//! The keyboard's way through the window: the sidebar, then the page's
//! header, then its body, the status bar last, and no stop without a name.

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::{self, NodeT as _, Queryable as _};
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};

use crate::common::launch_in;

fn app(page: PageId) -> Harness<'static, App> {
    let setup = Setup {
        open: Some(page),
        catalogs: crate::discover::source_catalogs(),
        ..Setup::default()
    };

    launch_in(toolbox_gui_kit::testing::harness(1100.0, 760.0), setup)
}

fn stops(harness: &mut Harness<'static, App>) -> Vec<String> {
    let mut seen = Vec::new();
    for _ in 0_u8..120 {
        harness.key_press(Key::Tab);
        harness.run();

        let focused = harness
            .query_all(kittest::by())
            .find(|node| node.accesskit_node().is_focused())
            .map(|node| node.accesskit_node().label().unwrap_or_default());

        let Some(name) = focused else { break };
        if seen.first() == Some(&name) {
            break;
        }

        seen.push(name);
    }

    seen
}

fn position(stops: &[String], name: &str) -> usize {
    stops
        .iter()
        .position(|stop| stop == name)
        .unwrap_or_else(|| panic!("{name} is no Tab stop: {stops:?}"))
}

#[test]
fn the_header_comes_before_the_body_and_the_status_bar_last() {
    let mut harness = app(PageId::Services);
    let stops = stops(&mut harness);

    let home = position(&stops, "Home");
    let settings = position(&stops, "Settings");
    assert!(
        home < settings,
        "the sidebar's foot before its list: {stops:?}"
    );

    let sidebar = position(&stops, "Services");
    let search = position(&stops, "Search services");
    let filter = position(&stops, "Hide disabled");

    assert!(sidebar < search, "the sidebar after the header: {stops:?}");
    assert!(search < filter, "the body before the header: {stops:?}");

    let tabs = position(&stops, "System");
    let refresh = position(&stops, "Read again");
    assert!(tabs < refresh, "refresh before the tabs: {stops:?}");
}

#[test]
fn every_tab_stop_has_a_name() {
    for page in [
        PageId::Home,
        PageId::Settings,
        PageId::Services,
        PageId::Maintenance,
    ] {
        let mut harness = app(page);
        let stops = stops(&mut harness);

        assert!(!stops.is_empty(), "{page:?} has no Tab stop");
        assert!(
            stops.iter().all(|name| !name.trim().is_empty()),
            "{page:?} has a Tab stop without a name: {stops:?}"
        );
    }
}

#[test]
fn the_status_bar_is_the_last_stop_of_the_window() {
    let (mut harness, _runtime, _answer, _reads) = crate::system::failed_units_bar();

    let stops = stops(&mut harness);
    let page = position(&stops, "Home");
    let bar = stops
        .iter()
        .rposition(|stop| stop == "Read again")
        .unwrap_or_else(|| panic!("the status bar has no Read again: {stops:?}"));

    assert!(page < bar, "the status bar before the sidebar: {stops:?}");
    assert_eq!(
        bar,
        stops.len() - 1,
        "stops after the status bar: {stops:?}"
    );
}

#[test]
fn a_rows_controls_take_tab_from_left_to_right() {
    let mut harness = app(PageId::Discover);
    let stops = stops(&mut harness);

    let menu = stops
        .iter()
        .position(|stop| stop.starts_with("More for "))
        .unwrap_or_else(|| panic!("no row menu among {stops:?}"));

    let app = stops[menu].trim_start_matches("More for ").to_owned();

    assert!(
        stops
            .iter()
            .rposition(|stop| *stop == app)
            .is_some_and(|switch| menu < switch),
        "the menu of {app} sits left of its switch and comes first"
    );
}

#[test]
fn after_apply_the_focus_moves_into_the_console() {
    let mut harness = crate::changes::app_with(&["firefox"]);
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Apply")
        .click();
    for _ in 0_u8..20 {
        harness.run();
    }

    let focused = harness
        .query_all(kittest::by())
        .find(|node| node.accesskit_node().is_focused())
        .and_then(|node| node.accesskit_node().label());

    assert!(
        focused
            .as_deref()
            .is_some_and(|name| name == "Cancel" || name == "Hide the Console"),
        "the focus is on {focused:?}"
    );
}
