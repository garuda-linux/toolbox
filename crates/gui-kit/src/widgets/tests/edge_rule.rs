//! The edge of a frameless control at rest: the stepped input fill inside a
//! 1 px card outline, as a field draws it.

use egui::{Color32, Theme, Ui};

use super::super::buttons::ButtonKind;
use super::super::{check_row, pill_button};
use crate::testing::{filled_rect, harness, rim_colours_around};
use crate::tokens::{self, Tokens};

/// Draws `build` in both themes and checks that the face `fill` picks out
/// is outlined by the card outline alone.
fn rests_in_outline(
    what: &str,
    build: impl Fn(&mut Ui) + Clone + 'static,
    fill: fn(&Tokens) -> Color32,
) {
    for theme in [Theme::Dark, Theme::Light] {
        let mut harness = harness(300.0, 120.0).build_ui(build.clone());
        harness.ctx.set_theme(theme);
        harness.run();

        let tokens = tokens::current(&harness.ctx);
        let face = filled_rect(&harness, fill(&tokens))
            .unwrap_or_else(|| panic!("{theme:?}: {what} has no filled face"));

        assert_eq!(
            rim_colours_around(&harness, face.center()),
            vec![tokens.card_outline],
            "{theme:?}: {what} at rest"
        );
    }
}

#[test]
fn a_soft_or_danger_button_at_rest_is_the_input_fill_inside_the_card_outline() {
    for kind in [ButtonKind::Soft, ButtonKind::Danger] {
        rests_in_outline(
            "push button",
            move |ui| _ = pill_button(ui, "Apply", kind),
            |tokens| tokens.input,
        );
    }
}

#[test]
fn a_primary_button_at_rest_is_its_grey_inside_the_card_outline() {
    rests_in_outline(
        "primary button",
        |ui| _ = pill_button(ui, "Apply", ButtonKind::Primary),
        |tokens| tokens.default_button,
    );
}

#[test]
fn an_unchecked_check_box_is_the_input_fill_inside_the_card_outline() {
    rests_in_outline(
        "unchecked box",
        |ui| _ = check_row(ui, false, "Kitty", ""),
        |tokens| tokens.input,
    );
}
