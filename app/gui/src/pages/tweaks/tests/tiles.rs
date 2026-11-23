//! The lead rows' glyphs, in the picture slot the catalog rows below them
//! have.

use egui::{Rect, vec2};
use toolbox_gui_kit::testing::{painted_text, textured_rects};

use super::{TweaksPage, shipped};
use crate::pages::rows::PICTURE_SIZE;
use crate::testing::assert_text_beside_picture;

const DNS: &str = "DNS server";
const DNS_DESCRIPTION: &str =
    "The servers that look up the addresses of websites for this computer.";
/// Kirigami's smallMedium icon, the size of a row's glyph.
const ICON: f32 = 22.0;

#[test]
fn a_lead_row_has_its_glyph_where_a_catalog_row_has_its_picture() {
    let mut harness = shipped().harness(TweaksPage::default());
    harness.run();

    let painted = painted_text(&harness, DNS).expect("the title is painted");
    let top = painted.pos.y;
    let glyph = textured_rects(&harness)
        .into_iter()
        .filter(|rect| (rect.width() - ICON).abs() < 0.5)
        .min_by(|one, other| {
            (one.top() - top)
                .abs()
                .total_cmp(&(other.top() - top).abs())
        })
        .expect("a glyph is painted");

    let slot = Rect::from_center_size(glyph.center(), vec2(PICTURE_SIZE, PICTURE_SIZE));

    assert_text_beside_picture(&harness, slot, (DNS, DNS_DESCRIPTION), DNS);
}
