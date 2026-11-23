use egui::accesskit::Role;
use egui::{Color32, Key, Theme, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::switch;
use super::super::switch::track_colours;
use crate::palette::Palette;
use crate::testing::{circle_fills, click_at, focused, harness, rim_colours_around};
use crate::tokens::{Tokens, contrast};
use crate::{UiExt as _, tokens};

/// The knob's fill and the faded field colour a disabled knob shows.
fn drawn_knob(in_disabled_ui: bool) -> (Color32, Color32) {
    let mut harness = harness(200.0, 80.0).build_ui(move |ui| {
        if in_disabled_ui {
            ui.disable();
        }

        ui.enabled_ui(false, |ui| switch(ui, false, "Wi-Fi"));
    });

    harness.ctx.set_theme(Theme::Dark);
    harness.run();

    let fills = circle_fills(&harness);
    let knob = *fills.last().expect("the switch has a knob");
    let alpha = harness.ctx.global_style().visuals.disabled_alpha;
    (
        knob,
        tokens::current(&harness.ctx).field.gamma_multiply(alpha),
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

/// An off track's edge reads on the window it sits on, so a switch shows
/// that it is off without its knob to tell.
#[test]
fn an_off_track_edge_reads_on_the_window_in_both_themes() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        let (_, rim) = track_colours(&tokens, false, false);

        let ratio = contrast(rim, tokens.window);
        assert!(ratio >= 3.0, "{theme:?}: {ratio}");
    }
}

fn off_switch() -> Harness<'static> {
    let mut harness = harness(200.0, 80.0).build_ui(|ui| {
        switch(ui, false, "Wi-Fi");
    });

    harness.run();
    harness
}

fn track_rim(harness: &Harness<'_>) -> Color32 {
    let centre = harness
        .get_by_role_and_label(Role::CheckBox, "Wi-Fi")
        .rect()
        .center();

    *rim_colours_around(harness, centre)
        .first()
        .expect("the switch has a track")
}

/// Only a key shows the focus (ui-rules.md). A click on the focused
/// switch keeps its focus, so the rim must rest once the pointer leaves.
#[test]
fn a_tabbed_switch_lights_its_rim_until_a_click() {
    let mut harness = off_switch();
    let tokens = tokens::current(&harness.ctx);

    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(track_rim(&harness), tokens.accent, "after Tab");

    let at = harness
        .get_by_role_and_label(Role::CheckBox, "Wi-Fi")
        .rect()
        .center();

    click_at(&mut harness, at);
    harness.hover_at(pos2(190.0, 70.0));
    harness.run();
    assert_eq!(
        focused(&harness).as_deref(),
        Some("Wi-Fi"),
        "the click keeps the focus"
    );
    assert_eq!(track_rim(&harness), tokens.rim, "after the click");
}
