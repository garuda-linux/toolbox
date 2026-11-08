//! One selection language: the accent tint of the sidebar, and in light
//! themes rims where a white surface sat on pale grey.

use egui::Theme;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{chip, segmented, switch};
use crate::testing::{filled_rect, harness, rims_around, text_colour};
use crate::tokens;

fn shown(theme: Theme, draw: impl FnMut(&mut egui::Ui) + 'static) -> Harness<'static> {
    let mut harness = harness(400.0, 120.0).build_ui(draw);
    harness.ctx.set_theme(theme);
    harness.run();
    harness
}

#[test]
fn a_selected_chip_takes_the_selection_tint() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = shown(theme, |ui| _ = chip(ui, "Audio", Some("3"), true));
        let tokens = tokens::current(&harness.ctx);

        assert!(
            filled_rect(&harness, tokens.accent_tint).is_some(),
            "{theme:?}: no tinted chip"
        );
        assert_eq!(text_colour(&harness, "Audio"), Some(tokens.on_tint));
        assert_eq!(text_colour(&harness, "3"), Some(tokens.on_tint));
    }
}

#[test]
fn an_off_switch_has_a_rim_in_light() {
    let harness = shown(Theme::Light, |ui| _ = switch(ui, false, "Bluetooth", true));

    let centre = harness.get_by_label("Bluetooth").rect().center();
    let rims = rims_around(&harness, centre);

    assert!(rims.iter().any(|width| *width >= 1.0), "{rims:?}");
}

#[test]
fn the_chosen_segment_has_a_rim_in_light() {
    let harness = shown(Theme::Light, |ui| {
        _ = segmented(ui, "Scope", &[(0_u8, "System"), (1, "User")], 0);
    });

    let centre = harness.get_by_label("System").rect().center();
    let rims = rims_around(&harness, centre);

    assert!(rims.iter().any(|width| *width >= 1.0), "{rims:?}");
}
