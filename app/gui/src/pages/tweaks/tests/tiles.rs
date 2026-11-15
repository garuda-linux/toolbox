//! The lead rows' glyph tiles, in the picture slot the catalog rows below
//! them have.

use toolbox_gui_kit::testing::{filled_rects, painted_text};
use toolbox_gui_kit::widgets::page_tile;

use super::{TweaksPage, shipped};
use crate::pages::rows::PICTURE_SIZE;
use crate::testing::assert_text_beside_picture;

const DNS: &str = "DNS server";
const DNS_DESCRIPTION: &str =
    "The servers that look up the addresses of websites for this computer.";

#[test]
fn a_lead_row_has_its_glyph_tile_where_a_catalog_row_has_its_picture() {
    let mut harness = shipped().harness(TweaksPage::default());
    harness.run();

    let painted = painted_text(&harness, DNS).expect("the title is painted");
    let top = painted.pos.y;
    let fill = page_tile(&harness.ctx).fill;
    let tile = filled_rects(&harness, fill)
        .into_iter()
        .filter(|rect| (rect.width() - PICTURE_SIZE).abs() < 0.5)
        .min_by(|one, other| {
            (one.top() - top)
                .abs()
                .total_cmp(&(other.top() - top).abs())
        })
        .expect("a glyph tile is painted");

    assert_text_beside_picture(&harness, tile, (DNS, DNS_DESCRIPTION), DNS);
}
