use egui::accesskit::Role;
use egui::{Key, Rect};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::App;
use toolbox_gui::page::PageId;
use toolbox_gui_kit::theme::SIDEBAR_WIDTH;
use toolbox_gui_kit::widgets::SIDE_INSET;

use super::{app, entry_name};
use crate::common::{HEIGHT, SMALLEST};

/// Kirigami's smallSpacing, between the search field and the toggle beside it.
const SEARCH_GAP: f32 = 4.0;

fn home_highlight(harness: &Harness<'_, App>) -> Rect {
    let home = entry_name(harness, PageId::Home);
    let row = harness.get_by_role_and_label(Role::Tab, &home).rect();
    row.shrink2(egui::vec2(SIDE_INSET, 0.0))
}

fn sidebar_search(harness: &Harness<'_, App>) -> Rect {
    harness
        .query_all_by_label("Search")
        .map(|node| node.rect())
        .find(|rect| rect.left() < SIDEBAR_WIDTH)
        .expect("the sidebar's search field")
}

#[test]
fn the_sidebar_names_no_app_above_its_search() {
    let harness = app(HEIGHT, false);

    let named = harness
        .query_all_by_label("Garuda Toolbox")
        .any(|node| node.rect().left() < SIDEBAR_WIDTH);

    assert!(!named);
}

/// The row's edges meet the entries also while a scrollbar takes room.
fn head_row_spans_the_entries(height: f32) {
    let harness = app(height, false);
    let highlight = home_highlight(&harness);
    let field = sidebar_search(&harness);
    let toggle = harness.get_by_label("Collapse sidebar").rect();

    assert_edges_meet([
        ("left", field.left(), highlight.left()),
        ("right", toggle.right(), highlight.right()),
        ("middle", toggle.center().y, field.center().y),
        ("gap", toggle.left() - field.right(), SEARCH_GAP),
    ]);
}

pub(super) fn assert_edges_meet<const N: usize>(edges: [(&str, f32, f32); N]) {
    let off: Vec<_> = edges
        .into_iter()
        .filter(|(_, one, other)| (one - other).abs() >= 0.5)
        .collect();

    assert!(off.is_empty(), "{off:?}");
}

#[test]
fn the_search_row_spans_the_entries() {
    head_row_spans_the_entries(HEIGHT);
}

#[test]
fn the_search_row_spans_the_entries_beside_a_scrollbar() {
    head_row_spans_the_entries(SMALLEST.1);
}

/// Whether a node named `label` holds the keyboard focus.
fn focused(harness: &Harness<'_, App>, label: &str) -> bool {
    harness
        .query_all_by_label(label)
        .any(|node| node.accesskit_node().is_focused())
}

#[test]
fn tab_reaches_the_search_field_before_the_collapse_toggle() {
    let mut harness = app(HEIGHT, false);

    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        focused(&harness, "Search"),
        "the field is not the first stop"
    );

    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        focused(&harness, "Collapse sidebar"),
        "the toggle does not follow the field"
    );
}

#[test]
fn the_search_hint_shows_whole_in_english_and_german() {
    for (language, hint) in [
        ("en-US", "Search everything"),
        ("de-DE", "Alles durchsuchen"),
    ] {
        let setup = toolbox_gui::Setup {
            system_language: language.parse().unwrap(),
            ..toolbox_gui::Setup::default()
        };

        let harness = crate::common::launch_in(crate::common::window(HEIGHT), setup);
        let sidebar = Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(SIDEBAR_WIDTH, HEIGHT));

        let shape = toolbox_gui_kit::testing::painted_text_within(&harness, hint, sidebar)
            .unwrap_or_else(|| panic!("{language}: the hint is painted"));

        assert!(!shape.galley.elided, "{language}: the hint is cut short");
    }
}
