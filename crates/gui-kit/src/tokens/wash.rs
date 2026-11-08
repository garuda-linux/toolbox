//! Washes of a status colour over the card, for banners, tags and the
//! tile of a status glyph.

use egui::Color32;

use super::{DIM_CONTRAST, Tile, Tokens, fitted, mix};

const BANNER_WASH: f32 = 0.12;
const TAG_WASH: f32 = 0.14;
/// How much of its status colour a status glyph's tile takes: twice the
/// banner's, so the tile still shows on the banner it marks.
const STATUS_TILE_WASH: f32 = 0.24;

impl Tokens {
    pub fn banner_wash(&self, colour: Color32) -> Color32 {
        mix(self.card, colour, BANNER_WASH)
    }

    pub fn tag_wash(&self, colour: Color32) -> Color32 {
        mix(self.card, colour, TAG_WASH)
    }

    /// The soft tile of a status glyph in `colour`. The glyph keeps the
    /// status colour, moved only where a desktop colour misses 3:1 on it.
    pub fn status_tile(&self, colour: Color32) -> Tile {
        let fill = mix(self.card, colour, STATUS_TILE_WASH);
        Tile {
            fill,
            ink: fitted(colour, &[fill], DIM_CONTRAST),
        }
    }
}
