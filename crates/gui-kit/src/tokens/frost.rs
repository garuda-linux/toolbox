//! The frosted surfaces' colours: as thin as the text on them allows.

use egui::{Color32, Theme};

use super::{SECONDARY_CONTRAST, contrast, mix};

/// The frost never turns fully opaque, or the blur would not show.
const FROST_MOST: f32 = 0.96;
/// The header's scrim and the terminal's glass take at least this much, so
/// what lies under them stays muted and the terminal's box shows, and no
/// more, so their blur shows as soft shapes.
pub(super) const SCRIM_LEAST: f32 = 0.2;
/// A frosted surface without a blur behind it lets no more than this
/// much of the sharp page through.
const VEIL_LEAST: f32 = 0.86;
const VEIL_MOST: f32 = 0.99;
const EXTREMES: [Color32; 2] = [Color32::WHITE, Color32::BLACK];

impl super::Surfaces {
    /// The header's scrim over the live blur, black in a dark theme and
    /// white in a light one, as thin as each text keeps 4.5:1 over the
    /// app's surfaces, so the blur shows.
    pub(super) fn scrim(&self, theme: Theme, texts: [Color32; 2]) -> Color32 {
        let neutral = neutral(theme);
        let surfaces = self.under_text();
        neutral.gamma_multiply(least_band(|alpha| {
            texts
                .iter()
                .all(|text| reads(*text, neutral, alpha, &surfaces))
        }))
    }

    /// The Console sheet's tint over the live blur: the pane as thin as
    /// `text` on it keeps 4.5:1 over the app's surfaces.
    pub(super) fn frost(&self, text: Color32, sidebar: Color32, accent: Color32) -> Color32 {
        thin(self.pane, text, &self.under_blur(sidebar, accent))
    }
}

/// The least neutral layer, in hundredths from [`SCRIM_LEAST`], that
/// `enough` accepts, or [`FROST_MOST`] when none does.
pub(super) fn least_band(enough: impl Fn(f32) -> bool) -> f32 {
    (0_u8..=100)
        .map(|step| f32::from(step) / 100.0)
        .filter(|alpha| *alpha >= SCRIM_LEAST - f32::EPSILON && *alpha <= FROST_MOST)
        .find(|alpha| enough(*alpha))
        .unwrap_or(FROST_MOST)
}

/// Black in a dark theme, white in a light one: a scrim of it casts no hue.
pub(super) const fn neutral(theme: Theme) -> Color32 {
    match theme {
        Theme::Dark => Color32::BLACK,
        Theme::Light => Color32::WHITE,
    }
}

/// A frosted surface with no blur behind it: never thinner than
/// [`VEIL_LEAST`], so sharp text under it stays a faint ghost.
pub(super) fn veil(surface: Color32, text: Color32) -> Color32 {
    let least = (0_u8..=13)
        .map(|step| VEIL_LEAST + f32::from(step) / 100.0)
        .find(|alpha| reads(text, surface, *alpha, &EXTREMES))
        .unwrap_or(VEIL_MOST);

    surface.gamma_multiply(least.min(VEIL_MOST))
}

/// `surface` as thin as `text` on it keeps 4.5:1 over each of `under`,
/// the app's own surfaces: the live blur only ever blurs those, and a
/// blur cannot get brighter or darker than what it mixes.
pub(super) fn thin(surface: Color32, text: Color32, under: &[Color32]) -> Color32 {
    let alpha = quarter_steps()
        .take_while(|alpha| *alpha <= FROST_MOST)
        .find(|alpha| reads(text, surface, *alpha, under))
        .unwrap_or(VEIL_MOST);

    surface.gamma_multiply(alpha)
}

fn reads(text: Color32, surface: Color32, alpha: f32, under: &[Color32]) -> bool {
    under
        .iter()
        .all(|backdrop| contrast(text, mix(*backdrop, surface, alpha)) >= SECONDARY_CONTRAST)
}

fn quarter_steps() -> impl Iterator<Item = f32> {
    (10_u8..=40).map(|step| f32::from(step) / 40.0)
}
