use std::cell::Cell;
use std::rc::Rc;

use egui::accesskit::Role;
use egui::{Rect, TextEdit, Theme, Ui};
use egui_kittest::kittest::Queryable as _;

use super::super::*;
use crate::testing::{filled_rect, harness};
use crate::tokens;

const NAME: &str = "The field";

/// The fill of a field sits this far inside the widget on each side
/// (`INSET` in `field.rs`).
const FILL_INSET: f32 = 2.0;

/// Every single-line control kind, each named [`NAME`].
const KINDS: [(&str, Kind); 7] = [
    ("text field", |ui| {
        let mut text = "Hello".to_owned();
        text_field(ui, TextEdit::singleline(&mut text)).accessible_name(NAME);
    }),
    ("text field with unit", |ui| {
        let mut text = "60".to_owned();
        text_field_with_unit(ui, TextEdit::singleline(&mut text), "min").accessible_name(NAME);
    }),
    ("number field", |ui| {
        _ = number_field(ui, &mut 5, 0..=10, 72.0, (NAME, NAME));
    }),
    ("search field", |ui| {
        let texts = SearchTexts {
            name: NAME,
            hint: "Name or description",
            clear: "Clear the search",
        };

        search_field(ui, "height-search", &mut String::new(), texts, 300.0);
    }),
    ("secret field", |ui| {
        let texts = SecretTexts {
            name: NAME,
            show: "Show password",
            hide: "Hide password",
        };

        secret_field(ui, &mut SecretText::with_room(32), texts);
    }),
    ("spin field", |ui| {
        let texts = SpinTexts {
            salt: NAME,
            name: NAME,
            unit: "min",
            less: "Less",
            more: "More",
        };

        _ = spin_field(ui, &mut 5, 0..=10, 120.0, texts);
    }),
    ("dropdown", |ui| {
        _ = Select::new(NAME, &["One", "Two"])
            .selected(Some(0))
            .show(ui);
    }),
];

/// One kind of single-line control, drawn in a ui.
type Kind = fn(&mut Ui);

/// The kind's height and the push button's in the same ui. A field's fill
/// sits inside its widget, so its height plus the inset on both sides is
/// the widget's.
fn heights(kind: fn(&mut Ui), theme: Theme) -> (f32, f32) {
    let pushed = Rc::new(Cell::new(0.0));
    let shared = Rc::clone(&pushed);
    let mut harness = harness(480.0, 200.0).build_ui(move |ui| {
        shared.set(push_height(ui));
        kind(ui);
    });

    harness.ctx.set_theme(theme);
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    let fill = filled_rect(&harness, tokens.input).expect("the control's input fill is painted");
    let height = 2.0_f32.mul_add(FILL_INSET, fill.height());

    (height, pushed.get())
}

#[test]
fn every_single_line_control_is_as_tall_as_a_push_button() {
    for theme in [Theme::Dark, Theme::Light] {
        for (name, kind) in KINDS {
            let (height, push) = heights(kind, theme);

            assert!(
                (height - push).abs() < 0.5,
                "{theme:?}: {name} is {height} px, a push button {push} px"
            );
        }
    }
}

fn rect_of(harness: &egui_kittest::Harness<'_, String>, role: Role, label: &str) -> Rect {
    harness.get_by_role_and_label(role, label).rect()
}

/// A search field and a push button in one row share their top and their
/// bottom edge, as in the Diagnostics toolbar.
#[test]
fn a_search_field_and_a_push_button_in_one_row_share_their_edges() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, text: &mut String| {
            ui.horizontal(|ui| {
                let texts = SearchTexts {
                    name: "Search the report",
                    hint: "Search the report",
                    clear: "Clear the search",
                };

                search_field(ui, "row-search", text, texts, 300.0);
                pill_button(ui, "Copy Report", ButtonKind::Primary);
            });
        },
        String::new(),
    );

    harness.run();

    let search = rect_of(&harness, Role::TextInput, "Search the report");
    let copy = rect_of(&harness, Role::Button, "Copy Report");
    let top_gap = (search.top() - copy.top()).abs();
    let bottom_gap = (search.bottom() - copy.bottom()).abs();

    assert!(
        top_gap.max(bottom_gap) < 0.5,
        "search {search:?} against the button {copy:?}"
    );
}
