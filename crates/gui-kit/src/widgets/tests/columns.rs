use std::cell::Cell;
use std::rc::Rc;

use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::columns::{Columns, MIN_COLUMN, Split, split};
use crate::testing::harness;
use crate::theme::LARGE_SPACING;

const GRID: f32 = 18.0;
/// The line between the two columns, and the gap on each side of it.
const LINE: f32 = 1.0;
const GAP: f32 = LARGE_SPACING;
/// The room the line and the two gaps take between the columns.
const BETWEEN: f32 = 2.0_f32.mul_add(GAP, LINE);

/// The narrowest content that holds two columns: two minimum columns, a gap
/// and the line on each side of it.
const NARROWEST: f32 = 2.0_f32.mul_add(MIN_COLUMN, BETWEEN);

/// The clip of each column and the page area, as the columns drew them.
#[derive(Clone, Copy, Debug)]
struct Drawn {
    area: Rect,
    list_clip: Rect,
    details_clip: Rect,
}

fn fits(width: f32) -> Split {
    split(width, GRID).expect("the content fits two columns")
}

#[test]
fn two_minimum_columns_with_their_gaps_and_the_line_are_the_least_that_fits() {
    assert_eq!(split(NARROWEST - 0.5, GRID), None);
    let least = fits(NARROWEST);
    assert_eq!(least.list, MIN_COLUMN);
    assert_eq!(least.details, MIN_COLUMN);
}

#[test]
fn the_list_takes_the_minimum_below_nine_hundred() {
    let split = fits(720.0);
    assert_eq!(split.list, MIN_COLUMN);
    assert_eq!(split.details, 720.0 - MIN_COLUMN - BETWEEN);
}

#[test]
fn the_list_takes_a_third_between_the_minimum_and_the_default() {
    let split = fits(1010.0);
    assert!((split.list - 1010.0 / 3.0).abs() < 1e-3);
    assert!((split.list + split.details + BETWEEN - 1010.0).abs() < 1e-3);
}

#[test]
fn the_list_stops_at_kirigami_default_column_width() {
    // Kirigami's default is 20 grid units, 360 points at 18.
    let split = fits(1600.0);
    assert_eq!(split.list, 360.0);
    assert_eq!(split.details, 1600.0 - 360.0 - BETWEEN);
}

#[test]
fn the_default_column_grows_with_the_grid_unit() {
    let split = split(1600.0, 24.0).expect("fits");
    assert_eq!(split.list, 480.0);
}

/// A list and its details, the list first, in a window `width` wide.
fn shown(width: f32, open: bool) -> Harness<'static> {
    let mut harness = harness(width, 300.0).build_ui(move |ui| {
        Columns::new("test").open(open).show(
            ui,
            &mut 0_u8,
            |ui, _| {
                ui.label("the list");
            },
            |ui, _| {
                ui.label("the details");
            },
        );
    });

    harness.run();
    harness
}

#[test]
fn a_wide_page_shows_the_list_and_the_details_side_by_side() {
    let harness = shown(900.0, true);

    let list = harness.get_by_label("the list").rect();
    let details = harness.get_by_label("the details").rect();
    assert!(list.right() < details.left(), "{list:?} {details:?}");
    assert!((list.top() - details.top()).abs() < 1.0);
}

#[test]
fn a_narrow_page_shows_the_list_alone_until_a_row_is_open() {
    let harness = shown(500.0, false);

    assert!(harness.query_by_label("the list").is_some());
    assert!(harness.query_by_label("the details").is_none());
}

#[test]
fn a_narrow_page_shows_the_details_alone_once_a_row_is_open() {
    let harness = shown(500.0, true);

    assert!(harness.query_by_label("the list").is_none());
    assert!(harness.query_by_label("the details").is_some());
}

#[test]
fn a_page_without_an_open_row_gives_the_list_the_full_width() {
    let harness = shown(900.0, false);

    assert!(harness.query_by_label("the list").is_some());
    assert!(harness.query_by_label("the details").is_none());
}

fn update(record: &Cell<Drawn>, change: impl FnOnce(&mut Drawn)) {
    let mut seen = record.get();
    change(&mut seen);
    record.set(seen);
}

/// Two columns at the content width `width`, the list drawn wider than any
/// column, with each column's clip recorded.
fn clips_at(width: f32) -> (Drawn, Harness<'static>) {
    let nothing = Rect::NOTHING;
    let drawn = Rc::new(Cell::new(Drawn {
        area: nothing,
        list_clip: nothing,
        details_clip: nothing,
    }));

    let record = Rc::clone(&drawn);
    let mut harness = harness(width, 300.0).build_ui(move |ui| {
        update(&record, |seen| seen.area = ui.max_rect());
        Columns::new("test").open(true).show(
            ui,
            &mut 0_u8,
            |ui, _| {
                update(&record, |seen| seen.list_clip = ui.clip_rect());
                let wide = Rect::from_min_size(ui.max_rect().min, egui::vec2(900.0, 20.0));
                ui.painter().rect_filled(wide, 0.0, egui::Color32::RED);
            },
            |ui, _| {
                update(&record, |seen| seen.details_clip = ui.clip_rect());
                ui.label("the details");
            },
        );
    });

    harness.run();
    (drawn.get(), harness)
}

/// At the narrowest width that holds two columns the list stays inside its
/// column, the details start after the gap and the line is between them.
#[test]
fn at_the_narrowest_two_column_width_the_list_is_clipped_and_gapped_from_the_details() {
    // The harness keeps 8 points of margin on each side of the content.
    let (drawn, harness) = clips_at(NARROWEST + 16.0);
    let widths = fits(drawn.area.width());
    let list_right = drawn.area.left() + widths.list;

    assert!(
        drawn.list_clip.right() <= list_right + 0.5,
        "list clip {:?} runs past its column at {list_right}",
        drawn.list_clip
    );
    assert!(
        drawn.details_clip.left() >= list_right + BETWEEN - 0.5,
        "details clip {:?} starts inside the gap",
        drawn.details_clip
    );
    let details = harness.get_by_label("the details").rect();
    assert!(
        details.left() >= drawn.details_clip.left() - 0.5,
        "{details:?} is clipped off"
    );
}
