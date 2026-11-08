//! The tinted squares behind row and section icons.
//!
//! Each part of the app has a hue of its own, so the open page's icons
//! share one colour. The shell sets it for the page each frame, rows never
//! pick their own.

use egui::{Color32, Context, Id, Theme};

use super::letters::LetterTiles;
use super::{DIM_CONTRAST, fitted, mix};
use crate::palette::{Palette, TileHues};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Hue {
    #[default]
    Accent,
    Blue,
    Green,
    Orange,
}

/// A tile's fill and the glyph's colour on it, at 3:1 (WCAG 1.4.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub fill: Color32,
    pub ink: Color32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tiles {
    accent: Tile,
    blue: Tile,
    green: Tile,
    orange: Tile,
    letters: LetterTiles,
}

impl Tiles {
    pub(super) fn derive(card: Color32, palette: &Palette, text: Color32, theme: Theme) -> Self {
        let hues = palette.tiles.unwrap_or_else(|| TileHues::of(theme));
        let tile = |hue: Color32| {
            let fill = mix(card, hue, fill_share(theme));
            Tile {
                fill,
                ink: fitted(hue, &[fill], DIM_CONTRAST),
            }
        };

        Self {
            accent: tile(palette.accent),
            blue: tile(hues.blue),
            green: tile(hues.green),
            orange: tile(hues.orange),
            letters: LetterTiles::derive(palette, text, theme),
        }
    }

    pub const fn of(&self, hue: Hue) -> Tile {
        match hue {
            Hue::Accent => self.accent,
            Hue::Blue => self.blue,
            Hue::Green => self.green,
            Hue::Orange => self.orange,
        }
    }

    pub const fn letters(&self) -> LetterTiles {
        self.letters
    }
}

pub fn set_page_hue(ctx: &Context, hue: Hue) {
    ctx.data_mut(|data| data.insert_temp(page_hue_id(), hue));
}

pub fn page_hue(ctx: &Context) -> Hue {
    ctx.data(|data| data.get_temp(page_hue_id()))
        .unwrap_or_default()
}

/// How much of its hue a tile takes over the card. Dark cards need more
/// for the square to show as much as on a white one.
const fn fill_share(theme: Theme) -> f32 {
    match theme {
        Theme::Dark => 0.20,
        Theme::Light => 0.16,
    }
}

fn page_hue_id() -> Id {
    Id::unique("toolbox_gui_kit::tokens::page_hue")
}
