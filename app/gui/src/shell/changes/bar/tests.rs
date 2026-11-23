use egui::{Rect, pos2};

use super::row_rect;

/// The page body's sides sit 18 points in from the page area's edges.
#[test]
fn the_row_starts_and_ends_on_the_page_body_edges() {
    let band = Rect::from_min_max(pos2(248.0, 672.0), pos2(1024.0, 720.0));

    let row = row_rect(band, 18.0);

    assert_eq!(row.left(), 266.0);
    assert_eq!(row.right(), 1006.0);
}
