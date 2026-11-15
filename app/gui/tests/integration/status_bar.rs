use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::{lines_in, text_colour};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::Tone;

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

    let text = harness.get_by_label(FAILED).rect();
    let tile = Tone::Warning.status_mark(&tokens).0.fill;
    let led = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, egui::Shape::Rect(rect)
            if rect.fill == tile
                && rect.rect.right() <= text.left() + 0.5
                && text.left() - rect.rect.right() <= 16.0
                && text.y_range().contains(rect.rect.center().y))
    });

    assert!(led, "no warning tile before {text:?}");
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
