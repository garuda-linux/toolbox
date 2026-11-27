use egui_kittest::kittest::Queryable as _;

use super::{page, plain_report, report};

#[test]
fn the_search_keeps_only_the_lines_that_hold_it() {
    let mut harness = page(report());

    harness.get_by_label("Search the report").click();
    harness.run();
    harness.get_by_label("Search the report").type_text("OOPS");
    harness.run();

    assert!(harness.query_by_label("kernel: oops").is_some());
    assert!(harness.query_by_label("kernel: hello").is_none());
}

#[test]
fn a_long_line_wraps_into_rows_that_fit_and_nothing_scrolls_sideways() {
    let harness = page(plain_report(format!("kernel: {}end\n", "x".repeat(400))));
    assert!(
        harness.query_by_label("Wrap lines").is_none(),
        "the report always wraps"
    );

    let report = harness.get_by_role(egui::accesskit::Role::Document).rect();
    let mut rows: Vec<egui::Rect> = Vec::new();

    for row in harness.query_all_by_label_contains("xxxx") {
        rows.push(row.rect());
    }

    assert!(rows.len() > 1, "the line did not wrap");
    assert!(
        rows.iter().all(|row| row.right() <= report.right()),
        "a wrapped row runs past the report's edge {}: {rows:?}",
        report.right()
    );
}

#[test]
fn the_rows_fold_while_a_report_shows_and_unfold_on_request() {
    let mut harness = page(report());
    assert!(harness.query_by_label("Collect Full report").is_none());

    harness.get_by_label("Collect Another Report").click();
    harness.run();

    assert!(harness.query_by_label("Collect Full report").is_some());
}
