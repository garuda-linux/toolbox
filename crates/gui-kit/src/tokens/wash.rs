//! Washes of a status colour over the card, for banners and the tile of a
//! status glyph.

use egui::Color32;

use super::readability::Ends;
use super::{DIM_CONTRAST, Tile, Tokens, fitted, mix};

const BANNER_WASH: f32 = 0.12;
/// How much of its status colour a status glyph's tile takes: twice
/// [`BANNER_WASH`], so the tile still shows on the wash it marks.
const STATUS_TILE_WASH: f32 = 0.24;

impl Tokens {
    pub fn banner_wash(&self, colour: Color32) -> Color32 {
        mix(self.card, colour, BANNER_WASH)
    }

    /// The soft tile of a status glyph in `colour`. The glyph keeps the
    /// status colour, moved only where a desktop colour misses 3:1 on it.
    pub fn status_tile(&self, colour: Color32) -> Tile {
        let fill = mix(self.card, colour, STATUS_TILE_WASH);
        let ends = Ends::of(self);

        Tile {
            fill,
            ink: fitted(colour, &[fill], DIM_CONTRAST, ends),
        }
    }
}
