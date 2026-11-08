//! Where painted text sits to the eye, and a font that tests it hard.

use egui::epaint::text::Glyph;
use egui::{Color32, Rect};
use egui_kittest::Harness;

use super::{painted_text, painted_text_within};
use crate::fonts::FontFile;

/// The deepest descent [`deep_descent_font`] has, in font units of its
/// 2048 to the em: more than twice Inter's own.
const DEEP_DESCENT: i16 = -1200;

/// How far below the middle of `rect` the last frame painted the middle
/// of `text`'s capitals, negative above it.
///
/// The `text` inside `rect` counts, else the one beside it. The middle
/// runs from a capital's real top to the baseline: the first capital
/// `text` holds, else an "H" in its font laid on its baseline.
pub fn cap_off_centre<State>(harness: &Harness<'_, State>, text: &str, rect: Rect) -> Option<f32> {
    let shape = painted_text_within(harness, text, rect).or_else(|| painted_text(harness, text))?;
    let (top, bottom) = capital_span(harness, &shape)?;
    Some(f32::midpoint(top, bottom) - rect.center().y)
}

/// Where the last frame painted the top of the capitals of `text`: its
/// first capital, else an "H" in its font laid on its baseline. `None`
/// when it painted no such text.
pub fn capitals_top<State>(harness: &Harness<'_, State>, text: &str) -> Option<f32> {
    let shape = painted_text(harness, text)?;
    capital_span(harness, &shape).map(|(top, _)| top)
}

/// The top and bottom of the capitals of the first line of `shape`, as
/// [`capitals_top`] finds them.
fn capital_span<State>(
    harness: &Harness<'_, State>,
    shape: &egui::epaint::TextShape,
) -> Option<(f32, f32)> {
    let row = shape.galley.rows.first()?;
    let origin = shape.pos.y + row.pos.y;

    let painted = row
        .glyphs
        .iter()
        .find(|glyph| glyph.chr.is_ascii_uppercase())
        .and_then(|glyph| glyph_span(&row.visuals.mesh, glyph));

    if let Some((top, bottom)) = painted {
        return Some((origin + top, origin + bottom));
    }

    let baseline = origin + row.glyphs.first()?.pos.y;
    let font = shape.galley.job.sections.first()?.format.font_id.clone();
    let capital = harness
        .ctx
        .fonts_mut(|fonts| fonts.layout_no_wrap("H".to_owned(), font, Color32::WHITE));

    let alone = capital.rows.first()?;
    let glyph = alone.glyphs.first()?;
    let (top, bottom) = glyph_span(&alone.visuals.mesh, glyph)?;
    let shift = baseline - glyph.pos.y;
    Some((top + shift, bottom + shift))
}

/// Where the last frame painted the baseline of the lowest line of text
/// inside `rect`, read from the foot of a letter that does not descend,
/// and the bottom of that line's box.
///
/// `None` when no such letter was painted there.
pub fn last_baseline_within<State>(harness: &Harness<'_, State>, rect: Rect) -> Option<(f32, f32)> {
    let lowest = super::painted(harness)
        .into_iter()
        .filter_map(|shape| {
            if let egui::Shape::Text(text) = shape {
                Some(text)
            } else {
                None
            }
        })
        .filter(|text| rect.contains(text.pos + text.galley.rect.center().to_vec2()))
        .max_by(|one, other| {
            let bottom = |text: &egui::epaint::TextShape| text.pos.y + text.galley.rect.bottom();
            bottom(one).total_cmp(&bottom(other))
        })?;

    let row = lowest.galley.rows.last()?;
    let glyph = row
        .glyphs
        .iter()
        .find(|glyph| glyph.chr.is_ascii_uppercase() || "acemnorsuvwxz".contains(glyph.chr))?;

    let (_, bottom) = glyph_span(&row.visuals.mesh, glyph)?;
    let line_bottom = lowest.pos.y + lowest.galley.rect.bottom();
    Some((lowest.pos.y + row.pos.y + bottom, line_bottom))
}

fn glyph_span(mesh: &egui::Mesh, glyph: &Glyph) -> Option<(f32, f32)> {
    let first = usize::try_from(glyph.first_vertex).ok()?;
    let quad = mesh.vertices.get(first..first.checked_add(4)?)?;
    let top = quad
        .iter()
        .map(|vertex| vertex.pos.y)
        .fold(f32::MAX, f32::min);

    let bottom = quad
        .iter()
        .map(|vertex| vertex.pos.y)
        .fold(f32::MIN, f32::max);

    Some((top, bottom))
}

/// The bundled Inter with a descent far deeper than its own, as some
/// desktop fonts have: egui's line box then sits well below the
/// capitals.
#[must_use = "the font does nothing until a harness uses it"]
pub fn deep_descent_font() -> FontFile {
    let mut data = include_bytes!("../../assets/fonts/InterVariable.ttf").to_vec();

    // Every place a font states its descent, as fonts pick one by flags.
    set(&mut data, *b"hhea", 6, DEEP_DESCENT.to_be_bytes());
    set(&mut data, *b"OS/2", 70, DEEP_DESCENT.to_be_bytes());
    set(
        &mut data,
        *b"OS/2",
        76,
        DEEP_DESCENT.unsigned_abs().to_be_bytes(),
    );

    FontFile {
        name: "Inter with a deep descent".to_owned(),
        data,
        index: 0,
        weight: 400,
    }
}

/// The fonts a centring test runs with: the bundled ones (`None`), the
/// [`deep_descent_font`], and every font file `TOOLBOX_TEST_FONTS` lists,
/// split by colons, such as a desktop's Noto Sans.
#[must_use = "the fonts do nothing until a harness uses them"]
pub fn test_fonts() -> Vec<Option<FontFile>> {
    let listed = std::env::var("TOOLBOX_TEST_FONTS").unwrap_or_default();
    let files = listed
        .split(':')
        .filter(|path| !path.is_empty())
        .filter_map(|path| {
            let data = std::fs::read(path).ok()?;
            Some(FontFile {
                name: path.to_owned(),
                data,
                index: 0,
                weight: 400,
            })
        });

    [None, Some(deep_descent_font())]
        .into_iter()
        .chain(files.map(Some))
        .collect()
}

fn set(data: &mut [u8], tag: [u8; 4], at: usize, value: [u8; 2]) {
    let start = table(data, tag).and_then(|start| start.checked_add(at));
    if let Some(slot) = start.and_then(|start| data.get_mut(start..start.checked_add(2)?)) {
        slot.copy_from_slice(&value);
    }
}

fn table(data: &[u8], tag: [u8; 4]) -> Option<usize> {
    let count = usize::from(u16::from_be_bytes(data.get(4..6)?.try_into().ok()?));
    let (records, _) = data.get(12..)?.as_chunks::<16>();
    records
        .iter()
        .take(count)
        .find(|record| record.starts_with(&tag))
        .and_then(|record| {
            let offset = u32::from_be_bytes(record.get(8..12)?.try_into().ok()?);
            usize::try_from(offset).ok()
        })
}
