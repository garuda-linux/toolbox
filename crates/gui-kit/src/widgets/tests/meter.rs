use egui::Rect;
use egui_kittest::Harness;

use super::super::{Tone, meter};
use crate::testing::{filled_rect, harness};
use crate::tokens;

fn shown(share: Option<f32>, tone: Tone) -> Harness<'static, Rect> {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        move |ui, rect: &mut Rect| *rect = meter(ui, share, tone),
        Rect::NOTHING,
    );

    harness.run();
    harness
}

#[test]
fn a_meter_fills_its_share_of_the_track() {
    let harness = shown(Some(0.25), Tone::Accent);
    let tokens = tokens::current(&harness.ctx);
    let line = *harness.state();

    let track = filled_rect(&harness, tokens.field).expect("a track");
    let filled = filled_rect(&harness, tokens.accent_ring).expect("a filled part");

    assert_eq!(track, line);
    assert!((track.height() - tokens.sizes.meter_height).abs() < 0.01);
    assert!((filled.left() - track.left()).abs() < 0.01);
    assert!(
        (filled.width() - track.width() / 4.0).abs() < 0.5,
        "{filled:?} of {track:?}"
    );
}

#[test]
fn a_meter_that_needs_a_look_fills_in_the_warning_tone() {
    let harness = shown(Some(0.95), Tone::Warning);
    let tokens = tokens::current(&harness.ctx);

    assert!(filled_rect(&harness, tokens.warning).is_some());
    assert!(filled_rect(&harness, tokens.accent_ring).is_none());
}

#[test]
fn without_a_share_the_line_keeps_its_height_and_shows_nothing() {
    let harness = shown(None, Tone::Accent);
    let tokens = tokens::current(&harness.ctx);

    assert!((harness.state().height() - tokens.sizes.meter_height).abs() < 0.01);
    assert!(filled_rect(&harness, tokens.field).is_none());
}

#[test]
fn a_tiny_share_still_shows_as_a_dot() {
    let harness = shown(Some(0.001), Tone::Accent);
    let tokens = tokens::current(&harness.ctx);
    let filled = filled_rect(&harness, tokens.accent_ring).expect("a filled part");

    assert!(filled.width() >= tokens.sizes.meter_height - 0.01);
}

#[test]
fn a_fine_meter_fills_in_the_accent_fitted_to_its_track() {
    let mut harness = harness(400.0, 100.0).build_ui(|ui| _ = meter(ui, Some(0.5), Tone::Accent));

    // The fallback accent fits already, a ring of its own tells them apart.
    let mut fitted = tokens::current(&harness.ctx);
    fitted.accent_ring = fitted.success;
    tokens::store(&harness.ctx, fitted, fitted);
    harness.run();

    assert!(filled_rect(&harness, fitted.success).is_some());
    assert!(filled_rect(&harness, fitted.accent).is_none());
}
