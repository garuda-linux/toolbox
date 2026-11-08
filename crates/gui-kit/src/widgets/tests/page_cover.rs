//! A page whose bottom lies under something floating over it, as the
//! Console's sheet does: the page paints on under it, and what the page
//! brings into sight lands above it.

use egui::Ui;
use egui_kittest::kittest::Queryable as _;

use super::page_layout;
use crate::testing::{harness, painted_text_within};

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 300.0;
const COVER: f32 = 150.0;
const ROWS: u8 = 30;
/// A row that starts under the cover.
const MIDDLE: u8 = 15;

/// A long page under a cover of [`COVER`]. The row the state names
/// scrolls itself into sight, once.
fn covered_page() -> egui_kittest::Harness<'static, Option<u8>> {
    let mut harness = harness(WIDTH, HEIGHT).build_ui_state(
        |root, reach: &mut Option<u8>| {
            super::super::cover_page_bottom(root.ctx(), COVER);
            page_layout(
                root,
                "covered",
                &mut (),
                |(), ui: &mut Ui| {
                    ui.label("Title");
                    true
                },
                |(), ui: &mut Ui| {
                    let wanted = reach.take();
                    for row in 0..ROWS {
                        let shown = ui.label(format!("Row {row}"));
                        if wanted == Some(row) {
                            shown.scroll_to_me(None);
                        }
                    }
                },
            );
        },
        None,
    );

    harness.run();
    harness
}

#[test]
fn the_page_paints_on_under_the_cover() {
    let harness = covered_page();
    let under =
        egui::Rect::from_min_max(egui::pos2(0.0, HEIGHT - COVER), egui::pos2(WIDTH, HEIGHT));

    let painted = (0..ROWS).any(|row| {
        let label = format!("Row {row}");
        harness.get_by_label(&label).rect().top() > HEIGHT - COVER
            && painted_text_within(&harness, &label, under).is_some()
    });

    assert!(painted, "no row under the cover was painted");
}

#[test]
fn a_row_scrolled_into_sight_lands_above_the_cover() {
    let mut harness = covered_page();
    let label = format!("Row {MIDDLE}");
    assert!(
        harness.get_by_label(&label).rect().bottom() > HEIGHT - COVER,
        "the row starts above the cover already"
    );

    *harness.state_mut() = Some(MIDDLE);
    harness.run();

    let row = harness.get_by_label(&label).rect();
    assert!(
        row.bottom() <= HEIGHT - COVER + 0.5,
        "the row ends at {} under a cover from {}",
        row.bottom(),
        HEIGHT - COVER
    );
}
