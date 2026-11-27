//! Text a widget cuts to fit ends in "…" and stays readable whole on
//! hover and for screen readers.

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotResults};

use super::super::{
    NavLook, Select, check_row, link_row_at_least, link_tile, menu_button, menu_item, nav_row,
};
use crate::icons;
use crate::testing::{harness, painted_text};

const LONG: &str = "Deutsch (Deutschland, mit sehr langer Beschreibung)";
const DETAIL: &str = "Check the system, clean the caches and repair what broke on the way";
const GERMAN: &str = "Paketzwischenspeicher leeren und verwaiste Pakete entfernen";
const NAV: NavLook = NavLook {
    expanded: true,
    height: 32.0,
};

/// Hovers `label` long enough for egui's tooltip delay.
fn hover<State>(harness: &mut Harness<'_, State>, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).hover();
    harness.run();
    harness.run_steps(90);
    harness.run();
}

fn select() -> Harness<'static> {
    let mut harness = harness(400.0, 300.0).build_ui(|ui| {
        let options = [LONG];
        _ = Select::new("Language", &options)
            .selected(Some(0))
            .width(140.0)
            .show(ui);
    });

    harness.run();
    harness
}

#[test]
fn a_select_cuts_a_long_choice_with_an_ellipsis() {
    let harness = select();

    let shown = painted_text(&harness, LONG).expect("the choice is painted");
    assert!(shown.galley.elided);
}

#[test]
fn a_select_shows_a_cut_choice_whole_on_hover() {
    let mut harness = select();

    hover(&mut harness, Role::ComboBox, "Language");

    assert!(
        harness.query_by_role_and_label(Role::Label, LONG).is_some(),
        "no tooltip with the whole choice"
    );
}

fn tile() -> Harness<'static> {
    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        _ = link_tile(ui, icons::GO_HOME, "Maintenance", DETAIL, 260.0);
    });

    harness.run();
    harness
}

#[test]
fn a_tile_tells_screen_readers_its_whole_line() {
    let harness = tile();

    let node = harness.get_by_role_and_label(Role::Link, "Maintenance");
    let description = node.accesskit_node().description();
    assert_eq!(description.as_deref(), Some(DETAIL));
}

#[test]
fn a_tile_shows_a_cut_line_whole_on_hover() {
    let mut harness = tile();

    hover(&mut harness, Role::Link, "Maintenance");

    assert!(
        harness
            .query_by_role_and_label(Role::Label, DETAIL)
            .is_some(),
        "no tooltip with the whole line"
    );
}

fn captioned() -> Harness<'static> {
    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        let options = ["Name", "Problems first"];
        ui.horizontal(|ui| {
            _ = Select::new("Sort units", &options)
                .caption("Sort by")
                .selected(Some(0))
                .width(160.0)
                .show(ui);
        });
    });

    harness.run();
    harness
}

#[test]
fn a_caption_names_a_select_to_the_eye_left_of_its_pill() {
    let harness = captioned();

    let caption = painted_text(&harness, "Sort by").expect("the caption is painted");
    let pill = harness.get_by_role_and_label(Role::ComboBox, "Sort units");
    assert!(caption.visual_bounding_rect().right() <= pill.rect().left());
}

#[test]
fn a_captioned_select_measures_its_caption_too() {
    let widths = std::rc::Rc::new(std::cell::Cell::new((0.0_f32, 0.0_f32)));
    let seen = std::rc::Rc::clone(&widths);
    let mut harness = harness(400.0, 200.0).build_ui(move |ui| {
        let options = ["Name"];
        let captioned = Select::new("Sort units", &options)
            .caption("Sort by")
            .width(160.0);

        let plain = Select::new("Sort", &options).width(160.0);
        seen.set((captioned.full_width(ui), plain.full_width(ui)));
    });

    harness.run();
    let (captioned, plain) = widths.get();
    assert!(captioned > 160.0 + 40.0, "caption not counted: {captioned}");
    assert!((plain - 160.0).abs() < f32::EPSILON);
}

#[test]
fn a_captioned_select_takes_no_more_than_its_full_width() {
    let widths = std::rc::Rc::new(std::cell::Cell::new((0.0_f32, 0.0_f32)));
    let seen = std::rc::Rc::clone(&widths);
    let mut harness = harness(400.0, 200.0).build_ui(move |ui| {
        let options = ["Name"];
        let select = Select::new("Sort units", &options)
            .caption("Sort by")
            .width(160.0);

        let full = select.full_width(ui);
        let used = ui
            .horizontal(|ui| {
                _ = select.show(ui);
            })
            .response
            .rect
            .width();

        seen.set((full, used));
    });

    harness.run();
    let (full, used) = widths.get();
    assert!((used - full).abs() < 0.5, "full {full}, used {used}");
}

/// A menu, a link row, a sidebar entry and a check row, each narrower
/// than its German label, the menu open.
fn labels(theme: Theme) -> Harness<'static> {
    let mut harness = harness(520.0, 360.0).with_theme(theme).build_ui(|ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(160.0);
                nav_row(ui, icons::GO_HOME, GERMAN, false, NAV);
                ui.add_space(12.0);
                _ = link_row_at_least(ui, icons::TOOLS, GERMAN, Role::Link, 0.0);
                ui.add_space(12.0);
                _ = check_row(ui, true, GERMAN, "");
            });

            ui.add_space(24.0);
            menu_button(ui, "Actions", |ui| {
                _ = menu_item(ui, GERMAN);
            });
        });
    });

    harness.run();
    harness.get_by_label("Actions").click();
    harness.run();
    harness
}

#[test]
fn painted_one_line_labels_cut_a_long_text() {
    let harness = labels(Theme::Dark);

    let cut: Vec<bool> = crate::testing::painted(&harness)
        .into_iter()
        .filter_map(|shape| {
            if let egui::Shape::Text(text) = shape
                && text.galley.text().starts_with("Paket")
            {
                Some(text.galley.elided)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(cut, [true; 4], "nav, link, check and menu labels");
}

#[test]
fn a_cut_menu_item_shows_whole_on_hover() {
    let mut harness = labels(Theme::Dark);

    hover(&mut harness, Role::MenuItem, GERMAN);

    assert!(
        harness
            .query_by_role_and_label(Role::Label, GERMAN)
            .is_some(),
        "no tooltip with the whole entry"
    );
}

#[test]
fn a_cut_link_row_shows_whole_on_hover() {
    let mut harness = labels(Theme::Dark);
    harness.key_press(egui::Key::Escape);
    harness.run();

    hover(&mut harness, Role::Link, GERMAN);

    assert!(
        harness
            .query_by_role_and_label(Role::Label, GERMAN)
            .is_some(),
        "no tooltip with the whole title"
    );
}

#[test]
fn cut_labels_in_both_themes() {
    let mut results = SnapshotResults::new();

    for (theme, name) in [
        (Theme::Dark, "widgets/cut_labels_dark"),
        (Theme::Light, "widgets/cut_labels_light"),
    ] {
        results.add(labels(theme).try_snapshot(name));
    }
}

#[test]
fn a_select_tells_its_choices_detail_to_screen_readers() {
    let mut harness = harness(400.0, 120.0).build_ui(|ui| {
        _ = Select::new("Login shell", &["fish"])
            .selected(Some(0))
            .detail(Some("/usr/bin/fish"))
            .show(ui);
    });

    harness.run();

    let select = harness.get_by_role_and_label(Role::ComboBox, "Login shell");

    assert_eq!(
        select.accesskit_node().description().as_deref(),
        Some("/usr/bin/fish")
    );
}

#[test]
fn a_tile_wraps_its_subtitle_to_two_lines_before_it_cuts() {
    const SETUP: &str = "Pick the apps you want and install them in one go";
    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        _ = link_tile(ui, icons::GO_HOME, "Setup Assistant", SETUP, 300.0);
    });

    harness.run();

    let tile = harness
        .get_by_role_and_label(Role::Link, "Setup Assistant")
        .rect();

    let shape = painted_text(&harness, SETUP).expect("the subtitle is painted");
    assert!(!shape.galley.elided, "the subtitle is cut");
    let text = shape.galley.rect.translate(shape.pos.to_vec2());
    assert_eq!(shape.galley.rows.len(), 2, "the subtitle takes two lines");
    assert!(
        tile.contains_rect(text),
        "the subtitle at {text:?} leaves the tile at {tile:?}"
    );
}
