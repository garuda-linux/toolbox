use egui::{Rect, Ui};
use egui_kittest::kittest::Queryable as _;

use super::super::{BodyShape, PageParts, page_layout_shaped};
use crate::testing::harness;

/// A page of `rows` rows with a footer that holds one button, in a
/// 400 × 300 window. Remembers the window's rect.
fn page(rows: u8) -> egui_kittest::Harness<'static, Rect> {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        move |root, window: &mut Rect| {
            *window = root.max_rect();
            let parts = PageParts {
                header: |(): &mut (), ui: &mut Ui| {
                    ui.label("Title");
                    true
                },
                body: |(): &mut (), ui: &mut Ui| {
                    for row in 0..rows {
                        ui.label(format!("Row {row}"));
                    }
                },
                footer: |(): &mut (), ui: &mut Ui| {
                    _ = ui.button("Next");
                    true
                },
            };

            page_layout_shaped(root, "page", &mut (), parts, BodyShape::Column);
        },
        Rect::NOTHING,
    );

    harness.run();
    harness
}

#[test]
fn the_footer_sits_at_the_bottom_of_a_short_page() {
    let harness = page(2);
    let window = *harness.state();
    let next = harness.get_by_label("Next").rect();
    let last = harness.get_by_label("Row 1").rect();
    let padding = f32::from(
        crate::tokens::current(&harness.ctx)
            .sizes
            .page_padding
            .bottom,
    );

    assert!(
        (window.bottom() - padding - next.bottom()).abs() < 1.0,
        "Next ends at {} in a window ending at {}",
        next.bottom(),
        window.bottom(),
    );
    assert!(
        next.top() > last.bottom() + 100.0,
        "the footer follows the rows"
    );
}

#[test]
fn the_footer_stays_in_place_on_a_long_page() {
    let harness = page(60);
    let window = *harness.state();
    let next = harness.get_by_label("Next").rect();

    assert!(next.bottom() <= window.bottom(), "Next is in view");

    let short = page(2).get_by_label("Next").rect();
    assert_eq!(next, short, "Next stays where a short page has it");
}

#[test]
fn a_footer_that_draws_nothing_takes_no_room() {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |root, window: &mut Rect| {
            *window = root.max_rect();
            let parts = PageParts {
                header: |(): &mut (), _: &mut Ui| false,
                body: |(): &mut (), ui: &mut Ui| {
                    for row in 0_u8..60 {
                        ui.label(format!("Row {row}"));
                    }
                },
                footer: |(): &mut (), _: &mut Ui| false,
            };

            page_layout_shaped(root, "page", &mut (), parts, BodyShape::Column);
        },
        Rect::NOTHING,
    );

    harness.run();

    let window = *harness.state();
    let lowest = (0_u8..60)
        .filter_map(|row| row_rect(&harness, row))
        .map(|row| row.bottom())
        .fold(f32::MIN, f32::max);

    assert!(lowest > window.bottom() - 30.0, "the rows reach the bottom");
}

fn row_rect<S>(harness: &egui_kittest::Harness<'static, S>, row: u8) -> Option<Rect> {
    let label = format!("Row {row}");
    harness.query_by_label(&label).map(|node| node.rect())
}
