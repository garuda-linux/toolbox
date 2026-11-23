use egui::accesskit::Role;
use egui::epaint::RectShape;
use egui::{Color32, TextEdit, Theme, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{
    SearchTexts, number_field, number_field_with_unit, search_field, text_box, text_field,
    text_field_with_unit,
};
use super::top_light::rects_within;
use crate::testing::{filled_rect, harness, rim_colours_around};
use crate::theme::FIELD_RADIUS;
use crate::tokens::{self, Tokens};

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
    |ui| _ = number_field(ui, &mut 5, 0..=10, 72.0, (NAME, NAME)),
    |ui| _ = number_field_with_unit(ui, &mut 5, 0..=10, 72.0, (NAME, NAME, "s")),
    |ui| {
        let texts = SearchTexts {
            name: NAME,
            hint: "Name or description",
            clear: "Clear the search",
        };

        search_field(ui, "test-search", &mut String::new(), texts, 300.0);
    },
    |ui| {
        _ = text_box(
            ui,
            (NAME, NAME),
            TextEdit::multiline(&mut "one\ntwo"),
            160.0,
        );
    },
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
        // A field's text is the node to focus, not a part around it.
        harness
            .query_all_by_label(NAME)
            .find(|node| {
                matches!(
                    node.accesskit_node().role(),
                    Role::TextInput | Role::MultilineTextInput | Role::SpinButton
                )
            })
            .expect("the field's text")
            .focus();
        harness.run();
    }

    harness
}

/// A field at rest is its input fill inside the card outline, the outline
/// the only frame a field draws.
#[test]
fn every_field_at_rest_is_the_input_fill_inside_the_card_outline() {
    for theme in [Theme::Dark, Theme::Light] {
        for (index, kind) in KINDS.into_iter().enumerate() {
            let harness = shown(kind, theme, false);
            let tokens = tokens::current(&harness.ctx);
            let field = filled_rect(&harness, tokens.input).expect("the field is filled");

            assert_eq!(
                rim_colours_around(&harness, field.center()),
                vec![tokens.card_outline],
                "{theme:?}: field {index} at rest"
            );
        }
    }
}

/// A field at rest casts the shadow shade at 40 of 255 with a Qt blur of 5
/// (darklyhelper.cpp:1059), and a focused one a Qt blur of 7
/// (darklyhelper.cpp:1064). egui reads twice the Qt radius.
#[test]
fn a_field_casts_darkly_shadow_at_rest_and_a_wider_one_when_focused() {
    for theme in [Theme::Dark, Theme::Light] {
        for (index, kind) in KINDS.into_iter().enumerate() {
            let rest = shown(kind, theme, false);
            let tokens = tokens::current(&rest.ctx);
            assert_eq!(
                blurs(&rest, tokens.control_shadow),
                vec![10.0],
                "{theme:?}: field {index} at rest"
            );

            let focused = shown(kind, theme, true);
            assert_eq!(
                blurs(&focused, tokens.control_shadow),
                vec![14.0],
                "{theme:?}: field {index} focused"
            );
        }
    }
}

/// A focused field keeps its View fill inside a Highlight halo at 60 %,
/// which shows one pixel out from the fill on each side (darklyhelper.cpp:1066).
#[test]
fn a_focused_field_keeps_its_fill_inside_a_highlight_halo() {
    for theme in [Theme::Dark, Theme::Light] {
        for (index, kind) in KINDS.into_iter().enumerate() {
            let harness = shown(kind, theme, true);
            let tokens = tokens::current(&harness.ctx);
            let fill = filled_rect(&harness, tokens.input).expect("the field is filled");
            let halo = filled_rect(&harness, halo_colour(&tokens)).expect("the halo");

            assert!(
                rim_colours_around(&harness, fill.center()).is_empty(),
                "{theme:?}: field {index} has a frame"
            );
            assert!(
                (halo.width() - fill.width() - 4.0).abs() < 0.01
                    && (halo.height() - fill.height() - 4.0).abs() < 0.01,
                "{theme:?}: field {index} halo is not one pixel out on each side"
            );
        }
    }
}

/// The fill takes the field radius at rest and one less under the halo,
/// the halo one more (darklyhelper.cpp:1046, 1066).
#[test]
fn a_field_takes_the_field_radius_and_its_halo_steps_out_from_it() {
    for (index, kind) in KINDS.into_iter().enumerate() {
        let rest = shown(kind, Theme::Dark, false);
        let tokens = tokens::current(&rest.ctx);
        assert_eq!(
            corners(&rest, tokens.input),
            Some(FIELD_RADIUS),
            "rest {index}"
        );

        let focused = shown(kind, Theme::Dark, true);
        let focused_tokens = tokens::current(&focused.ctx);
        assert_eq!(
            corners(&focused, focused_tokens.input),
            Some(FIELD_RADIUS - 1),
            "focused {index}"
        );
        assert_eq!(
            corners(&focused, halo_colour(&tokens)),
            Some(FIELD_RADIUS + 1),
            "halo {index}"
        );
    }
}

/// The Highlight at 60 % (darklyhelper.cpp:1066). The Highlight is the
/// selection's solid accent.
fn halo_colour(tokens: &Tokens) -> Color32 {
    tokens.accent_tint.gamma_multiply(0.6)
}

/// Every rectangle the last frame painted, a field's shapes inside their
/// group included.
fn painted_rects(harness: &Harness<'_>) -> Vec<RectShape> {
    harness
        .output()
        .shapes
        .iter()
        .flat_map(|clipped| rects_within(&clipped.shape))
        .collect()
}

/// The blur widths of the shadows the last frame painted in `fill`.
fn blurs(harness: &Harness<'_>, fill: Color32) -> Vec<f32> {
    painted_rects(harness)
        .iter()
        .filter(|rect| rect.fill == fill && rect.blur_width > 0.0)
        .map(|rect| rect.blur_width)
        .collect()
}

/// The corner radius of the first sharp rectangle the last frame filled
/// with `fill`.
fn corners(harness: &Harness<'_>, fill: Color32) -> Option<u8> {
    painted_rects(harness)
        .iter()
        .find(|rect| rect.fill == fill && rect.blur_width == 0.0)
        .map(|rect| rect.corner_radius.nw)
}
