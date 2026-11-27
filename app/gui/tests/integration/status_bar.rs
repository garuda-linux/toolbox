use egui_kittest::kittest::Queryable as _;
use toolbox_gui::testing::paints_icon;
use toolbox_gui_kit::icons::colour;
use toolbox_gui_kit::testing::{lines_in, text_colour};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::ICON_EDGE;

use crate::system::failed_units_bar;

const FAILED: &str = "Some parts of the system could not be read: system services";

#[test]
fn messages_side_by_side_are_parted_by_a_line() {
    let (mut harness, _runtime, _answer, _reads) = failed_units_bar();
    harness.run();

    let dry_run = harness
        .get_by_label_contains("Test mode: nothing is changed")
        .rect();

    let failed = harness.get_by_label(FAILED).rect();

    let outline = tokens::current(&harness.ctx).outline;

    let parting = lines_in(&harness, outline)
        .into_iter()
        .any(|[top, bottom]| {
            (top.x - bottom.x).abs() < 0.5 && top.x > dry_run.right() && top.x < failed.left()
        });

    assert!(parting, "no line between {dry_run:?} and {failed:?}");
}

#[test]
fn a_bar_message_leads_with_its_tile_in_plain_text() {
    let (mut harness, _runtime, _answer, _reads) = failed_units_bar();
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    assert_eq!(text_colour(&harness, FAILED), Some(tokens.text));

    // The mark is Breeze's warning icon, an image drawn in its own colours
    // (no tile behind it), so it is found by its place before the text and
    // told by the file its texture was loaded from.
    let text = harness.get_by_label(FAILED).rect();
    let before: Vec<_> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let egui::Shape::Rect(rect) = &clipped.shape else {
                return None;
            };

            (rect.brush.is_some()
                && rect.rect.right() <= text.left() + 0.5
                && text.left() - rect.rect.right() <= 16.0
                && text.y_range().contains(rect.rect.center().y))
            .then_some(rect.rect)
        })
        .collect();

    let [mark] = before[..] else {
        panic!("not one icon before {text:?}: {before:?}");
    };

    let edge = ICON_EDGE;
    assert!(
        (mark.width() - edge).abs() < 0.5 && (mark.height() - edge).abs() < 0.5,
        "{mark:?} is not the status icon's {edge}"
    );

    assert!(
        paints_icon(&harness, mark, colour::DIALOG_WARNING),
        "{mark:?} is not the warning icon"
    );
}

#[test]
fn a_hairline_parts_the_bar_from_the_page() {
    let (mut harness, _runtime, _answer, _reads) = failed_units_bar();
    harness.run();

    let text = harness.get_by_label(FAILED).rect();
    let outline = tokens::current(&harness.ctx).outline;
    let parted = lines_in(&harness, outline)
        .into_iter()
        .any(|[left, right]| {
            (left.y - right.y).abs() < 0.5
                && right.x - left.x > 400.0
                && left.y < text.top()
                && text.top() - left.y <= 24.0
        });

    assert!(parted, "no hairline on the bar above {text:?}");
}

#[test]
fn the_failed_probes_bar_in_light() {
    let (mut harness, _runtime, _answer, _reads) =
        crate::system::failed_units_bar_on(egui::Theme::Light);

    harness.run();
    harness.snapshot("status/failed_light");
}

#[test]
fn the_bar_is_frosted_over_the_live_blur() {
    let (mut harness, _runtime, _answer, _reads) = failed_units_bar();
    harness.run();

    let text = harness.get_by_label(FAILED).rect();
    let frost = tokens::current(&harness.ctx).frost;
    let textured = toolbox_gui_kit::testing::textured_rects(&harness);
    let frosted = toolbox_gui_kit::testing::filled_rects(&harness, frost)
        .iter()
        .any(|rect| rect.contains_rect(text) && textured.contains(rect));

    assert!(
        frosted,
        "the bar holds no frost over the blur around {text:?}"
    );
}
