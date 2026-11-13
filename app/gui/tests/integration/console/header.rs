use egui_kittest::kittest::Queryable as _;

use super::settle::{SETTLE, wait_for};
use super::{JAVA, app, succeeding_script};

#[test]
fn the_queued_count_is_centred_on_review_and_apply() {
    let (mut harness, _runtime) = app(succeeding_script(), JAVA);
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    // A plan that applied leaves the queue as it was here, so the count,
    // Review and Apply stay in the header.
    let count = harness.get_by_label("1 change queued").rect();
    let review = harness.get_by_label("Review").rect();

    assert!(
        (count.center().y - review.center().y).abs() < 0.5,
        "the count's middle is at {}, Review's at {}",
        count.center().y,
        review.center().y
    );
}

/// While the list is shut the sheet alone rises to the top, so a list
/// opened again from its header would paint under it.
#[test]
fn the_queue_list_opened_again_paints_above_the_console() {
    let (mut harness, _runtime) = app(succeeding_script(), JAVA);
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    for toggle in ["Review", "Hide", "Review"] {
        harness.get_by_label(toggle).click();
        harness.run_steps(SETTLE);
    }

    let title = harness.get_by_label("Queued changes").rect().center();
    let list = harness.ctx.layer_id_at(title).expect("the list's layer");
    let painted: Vec<egui::LayerId> = harness.ctx.memory(|memory| memory.layer_ids().collect());
    let at = |layer| painted.iter().position(|painted| *painted == layer);

    assert!(
        at(list) > at(toolbox_gui::testing::console_layer()),
        "the list paints before the Console: {painted:?}"
    );
}

/// The status mark and its word share the middle of the buttons beside
/// them, not the title's line.
#[test]
fn the_status_word_is_centred_on_review_and_apply() {
    let (mut harness, _runtime) = app(succeeding_script(), JAVA);
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    let status = harness.get_by_label("All changes applied").rect();
    let review = harness.get_by_label("Review").rect();

    assert!(
        (status.center().y - review.center().y).abs() < 0.5,
        "the status word's middle is at {}, Review's at {}",
        status.center().y,
        review.center().y
    );
}
