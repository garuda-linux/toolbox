use egui::{Color32, Theme};

use super::darkly::alpha;
use super::readability::Ends;
use super::{Surfaces, mix};

/// The accent pressed, the terminal's fill and the shadow. Black is the
/// theme's shadow shade and white its text, see [`Ends`].
pub(super) fn shades(
    theme: Theme,
    accent: Color32,
    surfaces: &Surfaces,
    ends: Ends,
) -> (Color32, Color32, Color32) {
    match theme {
        Theme::Dark => (
            mix(accent, ends.light(), 0.28),
            mix(surfaces.window, ends.shadow, 0.25),
            alpha(ends.shadow, 115),
        ),
        Theme::Light => (
            mix(accent, ends.shadow, 0.22),
            mix(surfaces.window, ends.text, 0.04),
            alpha(ends.shadow, 40),
        ),
    }
}

/// A light card carries a rim instead, so it keeps the card's own colour.
pub(super) fn top_light(theme: Theme, card: Color32, text: Color32) -> Color32 {
    match theme {
        Theme::Dark => mix(card, text, 0.1),
        Theme::Light => card,
    }
}

pub(super) fn shimmer(theme: Theme, bars: Color32, ends: Ends) -> Color32 {
    match theme {
        Theme::Dark => mix(bars, ends.light(), 0.08),
        Theme::Light => mix(bars, ends.shadow, 0.2),
    }
}
