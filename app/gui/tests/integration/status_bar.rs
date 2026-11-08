use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::lines_in;
use toolbox_gui_kit::tokens;

use crate::system::failed_units_bar;

#[test]
fn messages_side_by_side_are_parted_by_a_line() {
    let (mut harness, _runtime, _answer, _reads) = failed_units_bar();
    harness.run();

    let dry_run = harness
        .get_by_label_contains("Test mode: nothing is changed")
        .rect();

    let failed = harness
        .get_by_label("Some parts of the system could not be read: system services")
        .rect();

    let outline = tokens::current(&harness.ctx).outline;

    let parting = lines_in(&harness, outline)
        .into_iter()
        .any(|[top, bottom]| {
            (top.x - bottom.x).abs() < 0.5 && top.x > dry_run.right() && top.x < failed.left()
        });

    assert!(parting, "no line between {dry_run:?} and {failed:?}");
}
