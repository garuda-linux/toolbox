use egui_kittest::kittest::Queryable as _;

use super::settle::wait_for;
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
