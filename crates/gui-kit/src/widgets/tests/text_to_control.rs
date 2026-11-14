//! Every room between text and the controls beside it comes from one
//! token: a row's words and its actions, a page title and its tabs, a
//! property's label and its value.

use egui::accesskit::Role;
use egui::{Context, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{
    ActionRow, ButtonKind, PropertyTexts, Tone, page_header, pill_button, property_list, split_row,
};
use crate::icons;
use crate::testing::harness;
use crate::tokens::{self, Sizes};

/// Wide enough that no text fits beside a control in the test rows.
const WIDE: f32 = 400.0;

fn with_gap(ctx: &Context, gap: f32) {
    let mut current = tokens::current(ctx);
    current.sizes.text_to_control = gap;
    tokens::store(ctx, current, current);
}

fn drawn(gap: f32, body: impl FnMut(&mut Ui) + 'static) -> Harness<'static> {
    let mut harness = harness(700.0, 300.0).build_ui(body);
    with_gap(&harness.ctx, gap);
    harness.run();
    harness
}

fn button_under(harness: &Harness<'static>, text: &str) -> bool {
    let words = harness.get_by_label(text).rect();
    let button = harness
        .get_by_role_and_label(Role::Button, "Run now")
        .rect();

    button.top() >= words.bottom()
}

#[test]
fn an_action_row_keeps_the_gap_before_its_actions() {
    let row = |ui: &mut Ui| {
        ActionRow::new(icons::CHECKMARK, Tone::Neutral, "Update the system").show(ui, |ui| {
            _ = pill_button(ui, "Run now", ButtonKind::Soft);
        });
    };

    assert!(!button_under(
        &drawn(Sizes::DEFAULT.text_to_control, row),
        "Update the system"
    ));
    assert!(button_under(&drawn(WIDE, row), "Update the system"));
}

#[test]
fn a_split_row_keeps_the_gap_before_its_controls() {
    let row = |ui: &mut Ui| {
        split_row(
            ui,
            100.0,
            |ui| ui.label("Login shell"),
            |ui| pill_button(ui, "Run now", ButtonKind::Soft),
        );
    };

    assert!(!button_under(
        &drawn(Sizes::DEFAULT.text_to_control, row),
        "Login shell"
    ));
    assert!(button_under(&drawn(WIDE, row), "Login shell"));
}

#[test]
fn a_page_title_keeps_the_gap_before_its_controls() {
    let header = |ui: &mut Ui| {
        page_header(ui, "Services", "What runs and what starts.", 200.0, |ui| {
            _ = pill_button(ui, "Run now", ButtonKind::Soft);
        });
    };

    let under = |harness: &Harness<'static>| button_under(harness, "What runs and what starts.");
    assert!(!under(&drawn(Sizes::DEFAULT.text_to_control, header)));
    assert!(under(&drawn(WIDE, header)));
}

fn copy(label: &str) -> String {
    format!("Copy {label}")
}

#[test]
fn a_property_value_starts_the_gap_after_the_widest_label() {
    let gap = 40.0;
    let harness = drawn(gap, |ui| {
        let texts = PropertyTexts {
            copy: &copy,
            copied: "Copied",
        };

        property_list(ui, texts, |list| {
            list.row("Running since", "3 days");
        });
    });

    let label = harness.get_by_label("Running since").rect();
    let value = harness.get_by_label("3 days").rect();

    assert!(
        (value.left() - label.right() - gap).abs() < 0.5,
        "the value starts {} after its label",
        value.left() - label.right()
    );
}
