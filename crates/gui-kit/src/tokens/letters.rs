//! The tiles that stand in for a missing app picture, in hues taken from the
//! desktop's colour scheme.
//!
//! They start from [`crate::palette::letter_hues`] and move with the scheme's
//! own colours, so a muted scheme gives softer tiles.

use std::array;

use egui::{Color32, Theme};

use super::readability::Ends;
use super::{Tile, contrast, mix};
use crate::oklch::{from_oklch, hue_chroma, lightness, with_lightness};
use crate::palette::{Palette, letter_hues};

pub const LETTER_HUES: usize = 8;

/// What the letter reaches on its tile (WCAG AA for text).
const LETTER_CONTRAST: f32 = 4.5;
/// The least angle between two of the scheme's hues that both become
/// tiles. Closer ones read as the same colour.
const SCHEME_GAP: f32 = 30.0;
/// Below this OKLCH chroma a colour is grey and its hue says nothing.
const GREY_CHROMA: f32 = 0.03;
/// The turns of the accent the fill-up picks from, evenly round the wheel.
const TURNS: u16 = 24;
/// The share of the gap between the scheme's mean lightness and the
/// fallback's that the tiles follow. Status colours sit far lighter than
/// any tile a white letter reads on, and a lighter dark card such as
/// Nord's leaves little room below, so a larger share would overshoot.
const LIGHTNESS_SHARE: f32 = 0.25;
/// The most the tiles' lightness moves either way, so the letter still
/// reads and a tile still stands out from a dark card at 1.5:1.
const LIGHTNESS_REACH: f32 = 0.03;
/// The least and the most of the fallback's chroma the tiles take. Below
/// it the eight hues blur together, above it most hues leave sRGB at the
/// tiles' lightness and would differ in strength.
const CHROMA_SCALE: (f32, f32) = (0.5, 1.5);
/// How far a tile stands from the card at least (WCAG 1.4.11).
const TILE_APART: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LetterTiles {
    tiles: [Tile; LETTER_HUES],
}

impl LetterTiles {
    pub(super) fn derive(palette: &Palette, card: Color32, ends: Ends, theme: Theme) -> Self {
        let fills = if *palette == Palette::fallback(theme) {
            letter_hues(theme)
        } else {
            scheme_fills(palette, theme)
        };

        let fills = fills.map(|fill| tile_fill(fill, card, ends));

        let tiles = fills.map(|fill| Tile {
            fill,
            ink: ink_on(fill, ends),
        });

        Self { tiles }
    }

    pub fn of(&self, index: usize) -> Tile {
        let [first, ..] = self.tiles;
        self.tiles
            .get(index % LETTER_HUES)
            .copied()
            .unwrap_or(first)
    }
}

/// The scheme's hues first, in the order of [`scheme_hues`], then turns of
/// the accent, each at the lightness and chroma of [`softness`].
fn scheme_fills(palette: &Palette, theme: Theme) -> [Color32; LETTER_HUES] {
    let (light, chroma) = softness(palette, theme);
    let hues = scheme_hues(palette);

    hues.map(|hue| from_oklch(light, chroma, hue))
}

/// The lightness and chroma of every fill: the fallback fills', moved by
/// how far the mean of the scheme's colourful roles sits from the
/// fallback scheme's. A scheme of greys keeps the fallback's.
fn softness(palette: &Palette, theme: Theme) -> (f32, f32) {
    let [reference, ..] = letter_hues(theme);
    let (_, chroma) = hue_chroma(reference);
    let light = lightness(reference);

    let (Some((own_light, own_chroma)), Some((was_light, was_chroma))) = (
        mean_softness(palette),
        mean_softness(&Palette::fallback(theme)),
    ) else {
        return (light, chroma);
    };

    let lift = ((own_light - was_light) * LIGHTNESS_SHARE).clamp(-LIGHTNESS_REACH, LIGHTNESS_REACH);
    let (least, most) = CHROMA_SCALE;
    let scale = (own_chroma / was_chroma).clamp(least, most);

    (light + lift, chroma * scale)
}

/// The mean lightness and chroma of the roles of `palette` that are not
/// grey, none when all are.
fn mean_softness(palette: &Palette) -> Option<(f32, f32)> {
    let (count, light, chroma) = roles(palette)
        .filter_map(|colour| {
            let (_, chroma) = hue_chroma(colour);
            (chroma >= GREY_CHROMA).then(|| (lightness(colour), chroma))
        })
        .fold((0.0_f32, 0.0, 0.0), |(count, light, sum), (one, chroma)| {
            (count + 1.0, light + one, sum + chroma)
        });

    (count > 0.0).then_some((light / count, chroma / count))
}

/// The roles that carry the scheme's hues, in the order their hues lead.
fn roles(palette: &Palette) -> impl Iterator<Item = Color32> {
    [
        Some(palette.accent),
        Some(palette.success),
        Some(palette.warning),
        Some(palette.error),
        palette.link,
        palette.visited,
    ]
    .into_iter()
    .flatten()
}

/// Eight hue angles: the scheme's own, set apart, then the turn of the
/// accent farthest from every hue taken so far.
fn scheme_hues(palette: &Palette) -> [f32; LETTER_HUES] {
    let mut hues: Vec<f32> = Vec::with_capacity(LETTER_HUES);
    for (hue, chroma) in roles(palette).map(hue_chroma) {
        if chroma >= GREY_CHROMA && hues.iter().all(|taken| gap(*taken, hue) >= SCHEME_GAP) {
            hues.push(hue);
        }
    }

    let (accent, _) = hue_chroma(palette.accent);
    while hues.len() < LETTER_HUES {
        let next = farthest_turn(accent, &hues);
        hues.push(next);
    }

    array::from_fn(|index| hues.get(index).copied().unwrap_or(accent))
}

/// The turn of `accent` whose nearest taken hue is farthest away. The
/// first such turn wins a tie, so the accent itself leads when none is
/// taken yet.
fn farthest_turn(accent: f32, taken: &[f32]) -> f32 {
    let step = 360.0 / f32::from(TURNS);
    let room = |hue: f32| {
        taken
            .iter()
            .map(|other| gap(hue, *other))
            .fold(180.0_f32, f32::min)
    };

    (0..TURNS)
        .map(|turn| f32::from(turn).mul_add(step, accent).rem_euclid(360.0))
        .fold((accent, -1.0_f32), |(best, best_room), hue| {
            let here = room(hue);
            if here > best_room {
                (hue, here)
            } else {
                (best, best_room)
            }
        })
        .0
}

/// The angle between two hues, 0 to 180.
pub(super) fn gap(one: f32, other: f32) -> f32 {
    let turn = (one - other).rem_euclid(360.0);
    turn.min(360.0 - turn)
}

/// `fill`, darkened toward the shadow shade until the theme's text on it
/// reaches [`LETTER_CONTRAST`]. A turned hue at the same OKLCH lightness can
/// sit a little lighter in WCAG terms.
fn readable(fill: Color32, ends: Ends) -> Color32 {
    toward_shade(fill, ends.shadow)
        .find(|candidate| reads(*candidate, ends))
        .or_else(|| (!reads(ends.shadow, ends)).then(|| darker_of_its_hue(fill, ends))?)
        .unwrap_or(fill)
}

/// `fill` at the nearest lower OKLCH lightness its letter reads on, its hue
/// kept. KDE's shade of a near-white window is a mid grey that no letter
/// reads on, so the shadow shade cannot stand for black there.
fn darker_of_its_hue(fill: Color32, ends: Ends) -> Option<Color32> {
    let own = lightness(fill);
    (1_u8..=100)
        .map(|step| with_lightness(fill, own - f32::from(step) / 100.0))
        .find(|candidate| reads(*candidate, ends))
}

fn reads(fill: Color32, ends: Ends) -> bool {
    contrast(ink_on(fill, ends), fill) >= LETTER_CONTRAST
}

/// Whichever of the theme's text and window reads better on `fill`, so each
/// tile takes the ink its own hue needs.
fn ink_on(fill: Color32, ends: Ends) -> Color32 {
    if contrast(ends.text, fill) >= contrast(ends.window, fill) {
        ends.text
    } else {
        ends.window
    }
}

/// The fill of a tile, darkened toward the shadow shade until its letter
/// reads and until it stands [`TILE_APART`] from the card, within half way of
/// the shadow shade. Fallback and scheme tiles take the same darkening.
fn tile_fill(fill: Color32, card: Color32, ends: Ends) -> Color32 {
    let lettered = readable(fill, ends);
    let furthest = toward_shade(lettered, ends.shadow)
        .find(|candidate| contrast(*candidate, card) >= TILE_APART);

    furthest.unwrap_or_else(|| {
        toward_shade(lettered, ends.shadow)
            .max_by(|one, other| contrast(*one, card).total_cmp(&contrast(*other, card)))
            .unwrap_or(lettered)
    })
}

/// `fill` moved toward the shadow shade in steps, half way at the last.
fn toward_shade(fill: Color32, shadow: Color32) -> impl Iterator<Item = Color32> {
    (0_u8..=20).map(move |step| mix(fill, shadow, f32::from(step) / 40.0))
}
