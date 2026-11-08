//! Rows that keep to their height and line their text up, checked from
//! what screen readers see and what is painted.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui_kit::testing::{filled_rects, painted_text, textured_rects};
use toolbox_gui_kit::tokens::{self, LETTER_HUES};

use crate::pages::rows::PICTURE_SIZE;

/// Every row on screen is `height` tall and holds all its content, so no
/// text reaches past its row.
///
/// # Panics
///
/// When no row shows, or a row or a part of one differs from that.
pub fn assert_rows_hold_their_content<S>(harness: &Harness<'_, S>, height: f32) {
    assert_rows_hold(harness, Some(height));
}

/// Every row on screen, each as tall as its own content, holds all of
/// it, so no text reaches past its row.
///
/// # Panics
///
/// When no row shows, or a part of one reaches past its row.
pub fn assert_rows_hold_their_parts<S>(harness: &Harness<'_, S>) {
    assert_rows_hold(harness, None);
}

fn assert_rows_hold<S>(harness: &Harness<'_, S>, height: Option<f32>) {
    /// Rects land on fractions of a point.
    const SLACK: f64 = 0.5;

    let rows: Vec<_> = harness.query_all_by_role(Role::ListItem).collect();
    assert!(!rows.is_empty(), "no row on screen");
    for row in rows {
        let name = row.accesskit_node().label().unwrap_or_default();
        let Some(rect) = row.accesskit_node().bounding_box() else {
            continue;
        };

        let parts: Vec<(Option<String>, egui::accesskit::Rect)> = row
            .children_recursive()
            .filter_map(|part| {
                let node = part.accesskit_node();
                Some((node.label(), node.bounding_box()?))
            })
            .collect();

        if let Some(height) = height {
            assert!(
                (rect.height() - f64::from(height)).abs() < SLACK,
                "{name}: {} tall, not {height}. Its parts: {parts:#?}",
                rect.height()
            );
        }

        let room = egui::accesskit::Rect::new(
            rect.x0 - SLACK,
            rect.y0 - SLACK,
            rect.x1 + SLACK,
            rect.y1 + SLACK,
        );

        for (label, part) in &parts {
            assert!(
                room.contains(part.origin()) && room.contains((part.x1, part.y1).into()),
                "{label:?} at {part:?} reaches past the row {name} at {rect:?}"
            );
        }
    }
}

/// # Panics
///
/// When `text` is not painted.
pub fn text_left<S>(harness: &Harness<'_, S>, text: &str) -> f32 {
    let shape = painted_text(harness, text);
    assert!(shape.is_some(), "{text} is not painted");
    shape.map_or(f32::NAN, |shape| {
        let first = shape
            .galley
            .rows
            .first()
            .map_or(0.0, |line| line.rect().left());

        shape.pos.x + first
    })
}

/// A row's `title` and `description` start at one x, `picture_gap` right
/// of its `picture`.
///
/// # Panics
///
/// When either starts elsewhere.
pub fn assert_text_beside_picture<S>(
    harness: &Harness<'_, S>,
    picture: egui::Rect,
    (title, description): (&str, &str),
    what: &str,
) {
    /// Off by less than this, two edges count as one.
    const SAME: f32 = 0.5;

    let gap = tokens::Sizes::DEFAULT.picture_gap;
    let title = text_left(harness, title);
    let description = text_left(harness, description);
    let wanted = picture.right() + gap;

    assert!(
        (title - description).abs() < SAME,
        "{what}: the title starts at {title}, the description at {description}"
    );
    assert!(
        (title - wanted).abs() < SAME,
        "{what}: the title starts at {title}, {gap} right of the picture is {wanted}"
    );
}

/// The row picture, a letter tile or an image, level with `title`.
///
/// # Panics
///
/// When `title` is not painted or no picture sits beside it.
pub fn picture_beside<S>(harness: &Harness<'_, S>, title: &str) -> egui::Rect {
    /// How far past a picture the middle of a title's line may lie and
    /// still count as beside it.
    const REACH: f32 = 12.0;

    let shape = painted_text(harness, title);
    assert!(shape.is_some(), "{title} is not painted");
    let line = shape.map_or(f32::NAN, |shape| shape.pos.y + shape.galley.rect.center().y);

    let fills = tokens::current(&harness.ctx).tiles.letters();
    let picture = (0..LETTER_HUES)
        .flat_map(|hue| filled_rects(harness, fills.of(hue).fill))
        .chain(textured_rects(harness))
        .filter(|rect| (rect.width() - PICTURE_SIZE).abs() < 0.5)
        .find(|rect| (rect.top() - REACH..=rect.bottom() + REACH).contains(&line));

    assert!(picture.is_some(), "no picture beside {title}");
    picture.unwrap_or(egui::Rect::NOTHING)
}
