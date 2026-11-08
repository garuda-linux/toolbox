use egui::{Color32, Theme};
use egui_kittest::kittest::Queryable as _;

use super::super::switch;
use crate::testing::{harness, rim_colours_around};
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
