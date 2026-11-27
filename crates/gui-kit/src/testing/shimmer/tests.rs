use egui::Theme;

use super::shimmer_frozen;
use crate::testing::{harness, repaints_once_settled};
use crate::theme;
use crate::widgets::{SkeletonRows, card, skeleton_list};

/// Snapshots would catch the band wherever the frames a test ran left it,
/// so the harness stands the shimmer still and leaves other motion alone.
#[test]
fn a_test_harness_stands_the_shimmer_still() {
    let mut harness = harness(600.0, 400.0)
        .with_theme(Theme::Dark)
        .build_ui(|ui| {
            let rows = SkeletonRows {
                count: 3,
                row_height: 68.0,
                leading: true,
            };

            card(ui, |ui| skeleton_list(ui, rows, "Reading the services…"));
        });

    let repaints = repaints_once_settled(&mut harness);

    assert!(shimmer_frozen(&harness.ctx));
    assert!(repaints.is_empty(), "{repaints:?}");
    assert!(!theme::reduced_motion(&harness.ctx), "other motion stays");
}
