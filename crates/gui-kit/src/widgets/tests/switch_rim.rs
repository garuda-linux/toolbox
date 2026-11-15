use egui::{Color32, Theme};
use egui_kittest::kittest::Queryable as _;

use super::super::switch;
use crate::testing::{circle_fills, harness, rim_colours_around};
use crate::tokens::{self, contrast};

fn rim_contrast(theme: Theme, enabled: bool) -> f32 {
    let mut harness = harness(200.0, 80.0).build_ui(move |ui| {
        switch(ui, false, "Wi-Fi", enabled);
    });

    harness.ctx.set_theme(theme);
    harness.run();

    let centre = harness.get_by_label("Wi-Fi").rect().center();
    let rim: Color32 = *rim_colours_around(&harness, centre)
        .last()
        .expect("the switch has a rim");

    let card = tokens::current(&harness.ctx).card;
    contrast(card.blend(rim), card)
}

#[test]
fn a_disabled_switch_stays_seen_on_a_light_card() {
    let disabled = rim_contrast(Theme::Light, false);

    assert!(disabled >= 2.2, "the rim reads at {disabled}:1");
}

fn drawn_knob(in_disabled_ui: bool) -> (Color32, Color32) {
    let mut harness = harness(200.0, 80.0).build_ui(move |ui| {
        if in_disabled_ui {
            ui.disable();
        }

        switch(ui, false, "Wi-Fi", false);
    });

    harness.ctx.set_theme(Theme::Dark);
    harness.run();

    let fills = circle_fills(&harness);
    let knob = *fills.last().expect("the switch has a knob");
    let alpha = harness.ctx.global_style().visuals.disabled_alpha;
    (
        knob,
        tokens::current(&harness.ctx).text.gamma_multiply(alpha),
    )
}

#[test]
fn a_switch_in_a_disabled_ui_fades_once() {
    let (knob, once) = drawn_knob(true);

    assert_eq!(knob, once);
}

#[test]
fn a_switch_switched_off_on_its_own_fades_as_a_disabled_ui_does() {
    let (knob, once) = drawn_knob(false);

    assert_eq!(knob, once);
}

#[test]
fn a_disabled_switch_stays_dimmer_than_an_enabled_one() {
    for theme in [Theme::Dark, Theme::Light] {
        let disabled = rim_contrast(theme, false);
        let enabled = rim_contrast(theme, true);

        assert!(
            enabled - disabled >= 0.5,
            "{theme:?}: {disabled}:1 against {enabled}:1"
        );
    }
}
