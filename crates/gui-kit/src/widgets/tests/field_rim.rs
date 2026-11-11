use egui::accesskit::Role;
use egui::{Rect, TextEdit, Theme, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{
    SearchTexts, number_field, number_field_with_unit, search_field, text_box, text_field,
    text_field_with_unit,
};
use crate::testing::{filled_rect, harness, rim_colours_around};
use crate::tokens;

const NAME: &str = "The field";

/// Every kind of field, each named [`NAME`].
const KINDS: [fn(&mut Ui); 6] = [
    |ui| {
        let mut text = "Hello".to_owned();
        let response = text_field(ui, TextEdit::singleline(&mut text));
        named(ui, &response);
    },
    |ui| {
        let mut text = "60".to_owned();
        let response = text_field_with_unit(ui, TextEdit::singleline(&mut text), "min");
        named(ui, &response);
    },
    |ui| _ = number_field(ui, &mut 5, 0..=10, 72.0, NAME),
    |ui| _ = number_field_with_unit(ui, &mut 5, 0..=10, 72.0, (NAME, "s")),
    |ui| {
        let texts = SearchTexts {
            name: NAME,
            hint: "Name or description",
            clear: "Clear the search",
        };

        search_field(ui, &mut String::new(), texts, 300.0);
    },
    |ui| _ = text_box(ui, TextEdit::multiline(&mut "one\ntwo"), NAME, 160.0),
];

fn named(ui: &Ui, response: &egui::Response) {
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(NAME));
}

fn shown(kind: fn(&mut Ui), theme: Theme, focused: bool) -> Harness<'static> {
    let mut harness = harness(400.0, 300.0).build_ui(kind);
    harness.ctx.set_theme(theme);
    harness.run();
    if focused {
        // The search glyph carries the name too, as the label of its text.
        harness
            .query_all_by_label(NAME)
            .find(|node| {
                matches!(
                    node.accesskit_node().role(),
                    Role::TextInput | Role::MultilineTextInput
                )
            })
            .expect("the field's text")
            .focus();
        harness.run();
    }

    harness
}

fn fill_of(harness: &Harness<'_>, focused: bool) -> Rect {
    let tokens = tokens::current(&harness.ctx);
    let fill = if focused { tokens.hover } else { tokens.field };
    filled_rect(harness, fill).expect("the field is filled")
}

/// Only the fill marks a field at rest, as it looks while typing.
#[test]
fn every_field_at_rest_has_no_line_round_it() {
    for (index, kind) in KINDS.into_iter().enumerate() {
        let harness = shown(kind, Theme::Dark, false);
        let field = fill_of(&harness, false);

        assert_eq!(
            rim_colours_around(&harness, field.center()),
            [],
            "field {index}"
        );
    }
}

#[test]
fn a_focused_field_shows_only_the_focus_ring() {
    for (index, kind) in KINDS.into_iter().enumerate() {
        let harness = shown(kind, Theme::Dark, true);
        let field = fill_of(&harness, true);
        let tokens = tokens::current(&harness.ctx);

        assert_eq!(
            rim_colours_around(&harness, field.center()),
            vec![tokens.accent_ring],
            "field {index}"
        );
    }
}
