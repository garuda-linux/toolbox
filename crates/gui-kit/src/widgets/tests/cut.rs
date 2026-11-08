//! Text a widget cuts to fit ends in "…" and stays readable whole on
//! hover and for screen readers.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{Select, link_tile};
use crate::icons;
use crate::testing::{harness, painted_text};

const LONG: &str = "Deutsch (Deutschland, mit sehr langer Beschreibung)";
const DETAIL: &str = "Check the system, clean the caches and repair what broke on the way";

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
