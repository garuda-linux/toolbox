use egui::{Color32, Theme};

use super::{Surfaces, mix};

pub(super) fn shades(
    theme: Theme,
    accent: Color32,
    surfaces: &Surfaces,
    base: Color32,
    ink: Color32,
) -> (Color32, Color32, Color32, Color32) {
    match theme {
        Theme::Dark => (
            mix(accent, Color32::WHITE, 0.15),
            mix(accent, Color32::WHITE, 0.28),
            mix(surfaces.window, Color32::BLACK, 0.25),
            Color32::from_black_alpha(115),
        ),
        Theme::Light => (
            mix(accent, Color32::BLACK, 0.12),
            mix(accent, Color32::BLACK, 0.22),
            mix(base, ink, 0.04),
            Color32::from_black_alpha(40),
        ),
    }
}

pub(super) fn track_hover(theme: Theme, pressed: Color32, text: Color32) -> Color32 {
    match theme {
        Theme::Dark => pressed,
        Theme::Light => mix(pressed, text, 0.12),
    }
}

/// A light card carries a rim instead, so it keeps the card's own colour.
pub(super) fn top_light(theme: Theme, card: Color32) -> Color32 {
    match theme {
        Theme::Dark => mix(card, Color32::WHITE, 0.1),
        Theme::Light => card,
    }
}

pub(super) fn shimmer(theme: Theme, bars: Color32, card: Color32) -> Color32 {
    match theme {
        Theme::Dark => mix(bars, Color32::WHITE, 0.08),
        Theme::Light => mix(bars, card, 0.6),
    }
}
