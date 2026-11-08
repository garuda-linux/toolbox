//! Where painted things start, read from the rendered picture rather
//! than from layout boxes, which hold room above a glyph's ink.

use egui::Rect;
use egui_kittest::Harness;

/// The first pixel row inside `title` that holds light letters, light
/// text on a dark fill. `None` when the harness cannot render or `title`
/// holds no light pixel.
pub fn letters_top<State>(harness: &mut Harness<'_, State>, title: Rect) -> Option<f32> {
    letters(harness, title).map(|(top, _)| top)
}

pub fn letters<State>(harness: &mut Harness<'_, State>, title: Rect) -> Option<(f32, f32)> {
    let image = harness.render().ok()?;
    let scale = harness.ctx.pixels_per_point();
    let columns = pixel(title.left() * scale)..pixel(title.right() * scale);
    let rows = pixel(title.top() * scale)..pixel(title.bottom() * scale);

    let light = |y: u32| {
        columns.clone().any(|x| {
            let [red, green, blue, _] = image.get_pixel(x, y).0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 3 * 170
        })
    };

    let first = rows.clone().find(|y| light(*y))?;
    let last = rows.rev().find(|y| light(*y))?;
    Some((point(first, scale), point(last + 1, scale)))
}

/// The first pixel row inside `slot` that differs from the fill just
/// left of it, so a picture's transparent margin counts as that fill.
/// `None` when the harness cannot render or nothing is painted there.
pub fn drawing_top<State>(harness: &mut Harness<'_, State>, slot: Rect) -> Option<f32> {
    let image = harness.render().ok()?;
    let scale = harness.ctx.pixels_per_point();
    let columns = pixel(slot.left() * scale)..pixel(slot.right() * scale);
    let mut rows = pixel(slot.top() * scale)..pixel(slot.bottom() * scale);
    let fill = image
        .get_pixel(pixel((slot.left() - 4.0) * scale), rows.start)
        .0;

    let first = rows.find(|y| columns.clone().any(|x| image.get_pixel(x, *y).0 != fill))?;
    Some(point(first, scale))
}

fn point(row: u32, scale: f32) -> f32 {
    f32::from(u16::try_from(row).unwrap_or(u16::MAX)) / scale
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the test window is a few hundred whole pixels wide"
)]
const fn pixel(position: f32) -> u32 {
    position.round() as u32
}
