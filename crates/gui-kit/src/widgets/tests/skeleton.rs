use std::time::Duration;

use egui::epaint::Mesh;
use egui::{Shape, Theme};
use egui_kittest::Harness;

use super::super::{SkeletonRows, card, skeleton_list};
use super::motion::repaint_delay;
use crate::testing::{
    Cover, cover, freeze_shimmer, harness, lines_in, painted, repaints_once_settled,
};
use crate::{motion, theme, tokens};

const ROW_HEIGHT: f32 = 68.0;

fn rows(leading: bool) -> SkeletonRows {
    SkeletonRows {
        count: 3,
        row_height: ROW_HEIGHT,
        leading,
    }
}

fn shown(leading: bool) -> Harness<'static> {
    let rows = rows(leading);
    let mut harness = harness(600.0, 400.0)
        .with_theme(Theme::Dark)
        .build_ui(move |ui| {
            card(ui, |ui| skeleton_list(ui, rows, "Reading the services…"));
        });

    freeze_shimmer(&harness.ctx, false);
    harness.run();
    harness
}

fn swept(less_motion: bool) -> Harness<'static> {
    let mut harness = shown(true);
    theme::set_reduced_motion(&harness.ctx, less_motion);
    harness.input_mut().time = Some(f64::from(motion::SHIMMER_PERIOD) * 1.5_f64);
    harness.step();
    harness
}

/// The meshes painted, the card's top light left out.
fn bands<'frame, State>(harness: &'frame Harness<'_, State>) -> Vec<&'frame Mesh> {
    let light = tokens::current(&harness.ctx).top_light;
    painted(harness)
        .into_iter()
        .filter_map(|shape| {
            if let Shape::Mesh(mesh) = shape
                && mesh.vertices.iter().all(|vertex| vertex.color != light)
            {
                Some(&**mesh)
            } else {
                None
            }
        })
        .collect()
}

/// Square placeholders the size of an icon. The bars and controls are wider
/// than tall.
fn tiles(harness: &Harness<'_>) -> usize {
    painted(harness)
        .into_iter()
        .filter(|shape| {
            let Shape::Rect(rect) = shape else {
                return false;
            };

            let size = rect.rect.size();
            (size.x - size.y).abs() < 0.5 && (16.0..=24.0).contains(&size.y)
        })
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

#[test]
fn one_band_sweeps_every_row_at_one_place() {
    let harness = swept(false);
    let found = bands(&harness);
    let tops: Vec<f32> = found.iter().map(|mesh| mesh.calc_bounds().top()).collect();
    let (first, last) = (
        tops.iter().copied().fold(f32::INFINITY, f32::min),
        tops.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    );

    assert!(
        last - first > ROW_HEIGHT / 2.0,
        "the band crosses more than one row: {tops:?}"
    );

    // One phase for the whole list: wherever a bar is lit, its light is
    // the same at the same x.
    let lit: Vec<_> = found.iter().flat_map(|mesh| &mesh.vertices).collect();
    for one in &lit {
        for other in lit
            .iter()
            .filter(|other| (other.pos.x - one.pos.x).abs() < 0.1)
        {
            assert!(
                one.color.a().abs_diff(other.color.a()) <= 1,
                "at x {}: {} and {}",
                one.pos.x,
                one.color.a(),
                other.color.a()
            );
        }
    }

    let delay = repaint_delay(&harness);
    assert!(
        delay > Duration::ZERO && delay <= motion::SHIMMER_FRAME,
        "{delay:?}"
    );
}

#[test]
fn with_less_motion_the_bars_stand_still_and_ask_for_no_frames() {
    let harness = swept(true);

    assert_eq!(bands(&harness).len(), 0);
    assert_eq!(repaint_delay(&harness), Duration::MAX);
}

#[test]
fn a_skeleton_out_of_sight_asks_for_no_frames() {
    let mut harness = harness(600.0, 400.0).build_ui(|ui| {
        ui.add_space(1_000.0);
        skeleton_list(ui, rows(false), "Reading the services…");
    });

    freeze_shimmer(&harness.ctx, false);
    harness.run();
    harness.input_mut().time = Some(f64::from(motion::SHIMMER_PERIOD) * 1.5_f64);
    harness.step();

    assert_eq!(repaint_delay(&harness), Duration::MAX);
}

#[test]
fn a_skeleton_in_sight_asks_for_every_frame_of_its_sweep_and_none_with_less_motion() {
    for less_motion in [false, true] {
        let mut harness = shown(true);
        theme::set_reduced_motion(&harness.ctx, less_motion);
        let repaints = repaints_once_settled(&mut harness);

        assert_eq!(repaints.is_empty(), less_motion, "{repaints:?}");
    }
}

#[test]
fn a_skeleton_in_a_covered_window_asks_for_no_frames() {
    for how in [Cover::Minimized, Cover::Occluded] {
        let mut harness = shown(true);
        cover(&mut harness, how);
        let repaints = repaints_once_settled(&mut harness);

        assert!(repaints.is_empty(), "{how:?}: {repaints:?}");
    }
}

#[test]
fn a_skeleton_in_a_shown_window_without_the_focus_still_sweeps() {
    let mut harness = shown(true);
    harness.input_mut().focused = false;
    let repaints = repaints_once_settled(&mut harness);

    assert!(!repaints.is_empty());
}

/// Snapshots would catch the band wherever the frames a test ran left it.
#[test]
fn a_test_harness_stands_the_shimmer_still() {
    let mut harness = harness(600.0, 400.0).build_ui(|ui| {
        card(ui, |ui| {
            skeleton_list(ui, rows(true), "Reading the services…")
        });
    });

    let repaints = repaints_once_settled(&mut harness);

    assert!(repaints.is_empty(), "{repaints:?}");
    assert_eq!(bands(&harness).len(), 0);
    assert!(!theme::reduced_motion(&harness.ctx), "other motion stays");
}
