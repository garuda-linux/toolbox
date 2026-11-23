//! WCAG contrast, and the moves that bring a colour to a ratio on the fills
//! it sits on.

use egui::Color32;

use super::{Tokens, mix};
use crate::oklch::{lightness, with_lightness};

/// The steps of the whole OKLCH lightness scale a relit colour moves by.
const LIGHTNESS_STEPS: u8 = 100;

/// The theme's own colours a fitted colour moves between: its text and its
/// window, and the shadow shade that stands for black.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Ends {
    pub(super) text: Color32,
    pub(super) window: Color32,
    pub(super) shadow: Color32,
}

impl Ends {
    pub(super) const fn of(tokens: &Tokens) -> Self {
        Self {
            text: tokens.text,
            window: tokens.window,
            shadow: tokens.shadow_shade,
        }
    }

    /// Darkly's white: the lighter of the theme's text
    /// and window, the text in a dark theme and the window in a light one.
    pub(super) fn light(self) -> Color32 {
        if luminance(self.text) >= luminance(self.window) {
            self.text
        } else {
            self.window
        }
    }

    fn all(self) -> [Color32; 4] {
        [self.text, self.window, self.shadow, self.light()]
    }
}

pub fn contrast(first: Color32, second: Color32) -> f32 {
    let (lighter, darker) = {
        let (one, other) = (luminance(first), luminance(second));
        if one > other {
            (one, other)
        } else {
            (other, one)
        }
    };

    (lighter + 0.05) / (darker + 0.05)
}

/// The text of descriptions and hints. Page headers sit on the window,
/// which light themes draw darker than the cards, so it reads on both.
pub(super) fn description(
    text: Color32,
    card: Color32,
    window: Color32,
    ratio: f32,
    ends: Ends,
) -> Color32 {
    readable(
        readable(mix(text, card, 0.3), card, ratio, ends),
        window,
        ratio,
        ends,
    )
}

/// `color` moved until it reaches `ratio` on each of `fills`, which share
/// a side of the scale, so each move keeps the earlier ones.
pub(super) fn fitted(color: Color32, fills: &[Color32], ratio: f32, ends: Ends) -> Color32 {
    fills
        .iter()
        .fold(color, |moved, fill| readable(moved, *fill, ratio, ends))
}

/// `color` moved toward whichever of the theme's text, window, shadow shade
/// and light end reaches `ratio` on `background` in the fewest steps. When
/// none reaches it, the nearest OKLCH lightness of its own hue that does,
/// and when no lightness does either, the target that gets furthest.
pub(super) fn readable(color: Color32, background: Color32, ratio: f32, ends: Ends) -> Color32 {
    toward_ends(color, background, ratio, ends)
        .or_else(|| relit(color, background, ratio))
        .unwrap_or_else(|| {
            ends.all()
                .into_iter()
                .map(|target| mix(color, target, 1.0))
                .max_by(|one, other| {
                    contrast(*one, background).total_cmp(&contrast(*other, background))
                })
                .unwrap_or(color)
        })
}

/// `color` moved toward whichever end reaches `ratio` on `background` in
/// the fewest steps, none when no end does.
pub(super) fn toward_ends(
    color: Color32,
    background: Color32,
    ratio: f32,
    ends: Ends,
) -> Option<Color32> {
    let reached = ends.all().into_iter().filter_map(|target| {
        (0_u8..=20)
            .map(|step| (step, mix(color, target, f32::from(step) / 20.0)))
            .find(|(_, candidate)| contrast(*candidate, background) >= ratio)
    });

    reached
        .min_by_key(|(step, _)| *step)
        .map(|(_, candidate)| candidate)
}

/// `color` at the OKLCH lightness nearest its own that reaches `ratio` on
/// `background`, its hue kept. A theme whose lightest or darkest end sits
/// too close to its surfaces leaves the ends no colour that reads, so the
/// ink goes past them.
fn relit(color: Color32, background: Color32, ratio: f32) -> Option<Color32> {
    let own = lightness(color);
    (1_u8..=LIGHTNESS_STEPS)
        .map(|step| f32::from(step) / f32::from(LIGHTNESS_STEPS))
        .flat_map(|shift| [own + shift, own - shift])
        .filter(|level| (0.0..=1.0).contains(level))
        .map(|level| with_lightness(color, level))
        .find(|candidate| contrast(*candidate, background) >= ratio)
}

fn luminance(color: Color32) -> f32 {
    let linear = egui::Rgba::from(color);
    0.0722_f32.mul_add(
        linear.b(),
        0.2126_f32.mul_add(linear.r(), 0.7152 * linear.g()),
    )
}
