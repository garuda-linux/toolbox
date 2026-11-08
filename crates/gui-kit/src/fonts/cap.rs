//! Where a font's capitals sit in the line egui lays it out on.
//!
//! egui centres a line box: the ascent above the baseline, the descent
//! and the line gap below it. A font with a deep descent or a line gap
//! then shows its letters above the middle of a control, one with a tall
//! ascent below it. The eye reads a line's middle halfway up its
//! capitals, so each font's glyphs move by the difference.

#[cfg(test)]
mod tests;

use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider as _};

/// How far down to move the glyphs of a font, as a fraction of its size,
/// so the middle of a capital "H" sits on the middle of the line box.
///
/// `data` holds the font, `index` the face inside a collection. Negative
/// moves the glyphs up. A font that cannot be read, or has neither an
/// "H" nor a cap height, moves by 0.
pub fn cap_centring(data: &[u8], index: u32) -> f32 {
    let Ok(font) = FontRef::from_index(data, index) else {
        return 0.0;
    };

    // The metrics egui itself lays lines out with.
    let metrics = font.metrics(Size::unscaled(), LocationRef::default());
    let em = f32::from(metrics.units_per_em);
    let Some(cap_middle) = capital_middle(&font, metrics.cap_height).filter(|_| em > 0.0) else {
        return 0.0;
    };

    // From the line's top, in font units: the box's middle and the
    // capital's middle.
    let line = metrics.ascent - metrics.descent + metrics.leading;
    let shift = line / 2.0 - (metrics.ascent - cap_middle);
    shift / em
}

/// The height above the baseline of the middle of a capital: from the
/// "H" outline's own bounds, or half the font's stated cap height.
fn capital_middle(font: &FontRef<'_>, cap_height: Option<f32>) -> Option<f32> {
    let outline = font.charmap().map('H').and_then(|glyph| {
        font.glyph_metrics(Size::unscaled(), LocationRef::default())
            .bounds(glyph)
    });

    outline
        .filter(|bounds| bounds.y_max > bounds.y_min)
        .map(|bounds| f32::midpoint(bounds.y_max, bounds.y_min))
        .or_else(|| cap_height.filter(|cap| *cap > 0.0).map(|cap| cap / 2.0))
}
