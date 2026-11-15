use egui::Theme;
use egui_kittest::Harness;

use super::super::{SkeletonRows, card, skeleton_list};
use crate::testing::{filled_rects, harness, lines_in};
use crate::tokens;

const ROW_HEIGHT: f32 = 68.0;

fn shown(leading: bool) -> Harness<'static> {
    let rows = SkeletonRows {
        count: 3,
        row_height: ROW_HEIGHT,
        leading,
    };

    let mut harness = harness(600.0, 400.0)
        .with_theme(Theme::Dark)
        .build_ui(move |ui| {
            card(ui, |ui| skeleton_list(ui, rows, "Reading the services…"));
        });

    harness.run();
    harness
}

/// Square placeholders as tall as a row's tile, the bars and controls
/// are all wider than tall.
fn tiles(harness: &Harness<'_>) -> usize {
    let tokens = tokens::current(&harness.ctx);
    let (fill, edge) = (tokens.hover, tokens.sizes.icon_tile);
    filled_rects(harness, fill)
        .into_iter()
        .filter(|rect| (rect.width() - edge).abs() < 0.5 && (rect.height() - edge).abs() < 0.5)
        .count()
}

#[test]
fn rows_are_parted_by_a_divider_as_the_rows_that_come() {
    let harness = shown(false);
    let outline = tokens::current(&harness.ctx).outline;

    assert_eq!(lines_in(&harness, outline).len(), 2);
}

#[test]
fn only_leading_rows_hold_a_tile() {
    assert_eq!(tiles(&shown(false)), 0);
    assert_eq!(tiles(&shown(true)), 3);
}
