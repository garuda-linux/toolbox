//! Printed colours move in lightness until they read on the glass and without it.

use egui::{Color32, Theme};
use toolbox_terminal::Color;

use super::frost::{SCRIM_LEAST, least_band, neutral};
use super::{SECONDARY_CONTRAST, Surfaces, contrast, mix};
use crate::oklch::{lightness, with_lightness};
use crate::palette::Palette;
use crate::terminal_colors::{Ansi, PRINTED, resolve};

/// How far a printed colour moves in OKLCH lightness per step.
const LIGHTNESS_STEP: f32 = 0.005;
/// Steps from any lightness to black or white.
const LIGHTNESS_STEPS: u16 = 200;

impl Surfaces {
    /// The printed colours are moved in lightness until they read on the solid `terminal` and on the thinnest glass.
    pub(super) fn terminal_look(
        &self,
        theme: Theme,
        palette: &Palette,
        [text, terminal]: [Color32; 2],
        sidebar: Color32,
    ) -> (Color32, Ansi) {
        let frost = self.frost(text, sidebar, palette.accent);
        let under = self
            .under_blur(sidebar, palette.accent)
            .map(|own| own.blend(frost));

        let neutral = neutral(theme);
        let view = Palette {
            view: terminal,
            ..*palette
        };

        // A thicker glass only moves the backdrop further from the text,
        // so what reads on the thinnest reads on any.
        let thinnest = under.map(|backdrop| backdrop.blend(neutral.gamma_multiply(SCRIM_LEAST)));
        let ansi = Ansi::of(&view, theme == Theme::Dark)
            .fitted(|colour| lit(colour, theme, terminal, &thinnest));

        let inks: Vec<Color32> = std::iter::once(view.text)
            .chain(PRINTED.map(|index| resolve(Color::Indexed(index), &ansi, &view)))
            .collect();

        let glass = least_band(|alpha| {
            inks.iter().all(|ink| {
                under.iter().all(|backdrop| {
                    contrast(*ink, mix(*backdrop, neutral, alpha)) >= SECONDARY_CONTRAST
                })
            })
        });

        (neutral.gamma_multiply(glass), ansi)
    }
}

/// Until it reaches 4.5:1 on the terminal and on each of `under`. Hue and chroma stay as far as sRGB holds them.
fn lit(colour: Color32, theme: Theme, terminal: Color32, under: &[Color32]) -> Color32 {
    let reads = |candidate: Color32| {
        std::iter::once(&terminal)
            .chain(under)
            .all(|backdrop| contrast(candidate, *backdrop) >= SECONDARY_CONTRAST)
    };

    let (step, end) = match theme {
        Theme::Dark => (LIGHTNESS_STEP, 1.0),
        Theme::Light => (-LIGHTNESS_STEP, 0.0),
    };

    let start = lightness(colour);
    std::iter::once(colour)
        .chain(
            (1..=LIGHTNESS_STEPS)
                .map(|count| with_lightness(colour, f32::from(count).mul_add(step, start))),
        )
        .find(|candidate| reads(*candidate))
        .unwrap_or_else(|| with_lightness(colour, end))
}
