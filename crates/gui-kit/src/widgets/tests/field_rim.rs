use egui::accesskit::Role;
use egui::{Rect, TextEdit, Theme, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{
    SearchTexts, number_field, number_field_with_unit, search_field, text_box, text_field,
    text_field_with_unit,
};
use crate::testing::{filled_rect, harness, lines_in, rim_colours_around, rims_in};
use crate::tokens::{self, contrast};

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

#[test]
fn every_field_at_rest_has_a_thin_rim_round_its_whole_shape() {
    for (index, kind) in KINDS.into_iter().enumerate() {
        let harness = shown(kind, Theme::Dark, false);
        let field = fill_of(&harness, false);
        let rim = tokens::current(&harness.ctx).field_rim;
        let rims = rims_in(&harness, rim);

        assert_eq!(rims.len(), 1, "field {index}: {rims:?}");
        let (rect, width) = rims[0];
        assert!(rect.distance_to_pos(field.min) < 0.5, "field {index}");
        assert!(rect.distance_to_pos(field.max) < 0.5, "field {index}");
        assert!(width > 0.0 && width <= 1.0, "field {index}: {width}");
        assert!(lines_in(&harness, rim).is_empty(), "field {index}");
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
        assert!(
            lines_in(&harness, tokens.field_rim).is_empty(),
            "field {index}"
        );
    }
}

#[test]
fn the_rim_reads_three_to_one_on_cards_dialogs_the_window_and_its_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = shown(KINDS[0], theme, false);
        let tokens = tokens::current(&harness.ctx);
        let field = fill_of(&harness, false);
        let [rim] = rim_colours_around(&harness, field.center())[..] else {
            panic!("{theme:?}: one rim");
        };

        for (name, under) in [
            ("card", tokens.card),
            ("dialog", tokens.raised),
            ("window", tokens.window),
            ("fill", tokens.field),
        ] {
            let ratio = contrast(rim, under);
            assert!(ratio >= 3.0, "{theme:?} on the {name}: {ratio:.2}:1");
        }
    }
}
