//! The tile that stands in for an app's missing picture: the initial of
//! the name the row shows on a rounded square in one of the
//! [`crate::tokens::LetterTiles`].
//!
//! The initial follows the shown name, so it matches what the user reads.
//! The hue follows the package name, so an app keeps its colour on every
//! page and in every language. The tile is decorative, the row's own
//! label names the app to a screen reader.

use egui::{CornerRadius, FontId, Rect, Ui};

use crate::fonts::Weight;
use crate::tokens::{self, LETTER_HUES};

/// How much of the tile's edge the initial's font size takes.
const LETTER_SHARE: f32 = 0.5;

/// The first letter or digit of `shown`, upper case, or none when it has
/// neither. An upper case that spells two letters, as for "ß", keeps the
/// first.
pub fn initial(shown: &str) -> Option<char> {
    shown
        .chars()
        .find(|each| each.is_alphanumeric())
        .map(|first| first.to_uppercase().next().unwrap_or(first))
}

/// The hue of the tile of `package`, from 0 to [`LETTER_HUES`].
///
/// FNV-1a, as the standard hasher may change between Rust releases and would
/// recolour every tile with them. Its low bits alone spread short names
/// unevenly, so the high half is folded into them.
pub fn hue_of(package: &str) -> usize {
    let hash = package.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    });

    usize::try_from(hash ^ (hash >> 16)).unwrap_or_default() % LETTER_HUES
}

/// Paints the tile of `package` into `at` with corner `radius`, showing
/// the initial of `shown`, or of `package` when `shown` has none.
pub fn paint_letter_tile(ui: &Ui, at: Rect, radius: u8, shown: &str, package: &str) {
    let tile = tokens::current(ui.ctx())
        .tiles
        .letters()
        .of(hue_of(package));

    let painter = ui.painter();
    painter.rect_filled(at, CornerRadius::same(radius), tile.fill);

    let Some(letter) = initial(shown).or_else(|| initial(package)) else {
        return;
    };

    let font = FontId::new(at.height() * LETTER_SHARE, Weight::SemiBold.family());
    let galley = painter.layout_no_wrap(letter.to_string(), font, tile.ink);

    // Centred on its ink rather than its line box, which keeps room for
    // descenders and would set a capital high.
    let offset = at.center() - galley.mesh_bounds.center();
    painter.galley(offset.to_pos2(), galley, tile.ink);
}
