use egui::accesskit::{Role, Toggled};
use egui::{Color32, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::tristate_switch;
use crate::palette::Palette;
use crate::testing::{circle_fills, harness};
use crate::tokens::Tokens;

fn knob(theme: Theme, on: Option<bool>) -> Color32 {
    let mut harness = harness(200.0, 80.0).build_ui(move |ui| {
        tristate_switch(ui, on, "Wi-Fi");
    });

    harness.ctx.set_theme(theme);
    harness.run();

    *circle_fills(&harness)
        .last()
        .expect("the switch has a knob")
}

/// Breeze's knob is the field colour, on and off, in both themes.
#[test]
fn the_knob_is_the_field_colour_on_and_off() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        assert_eq!(knob(theme, Some(true)), tokens.field, "{theme:?}: on");
        assert_eq!(knob(theme, Some(false)), tokens.field, "{theme:?}: off");
    }
}

#[test]
fn an_unknown_switch_says_mixed_and_rests_its_knob_in_the_middle() {
    let mut harness = harness(200.0, 80.0).build_ui(|ui| {
        tristate_switch(ui, None, "Wi-Fi");
    });

    harness.run();
    let node = harness.get_by_role_and_label(Role::CheckBox, "Wi-Fi");
    assert_eq!(node.accesskit_node().toggled(), Some(Toggled::Mixed));

    let mut centre = None;
    for shape in crate::testing::painted(&harness) {
        if let egui::Shape::Circle(circle) = shape {
            centre = Some(circle.center);
        }
    }

    let rect = node.rect();
    let centre = centre.expect("the switch has a knob");
    assert!(
        (centre.x - rect.center().x).abs() < 0.5,
        "{centre:?} in {rect:?}"
    );
}
