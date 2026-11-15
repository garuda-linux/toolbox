//! The viewer's search keeps the lines that hold it, Wrap cuts a long line
//! at the edge, and the rows fold while a report shows.

use egui_kittest::kittest::Queryable as _;
use toolbox_services::diagnostics::page::{Page, State};

use super::{page, report};

fn with_long_line() -> Page {
    let long = format!("kernel: {}end\n", "x".repeat(400));
    Page {
        state: State::Report,
        selection: "dmesg".to_owned(),
        report: long.clone(),
        raw: long,
        ..Page::default()
    }
}

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
fn wrap_cuts_a_long_line_into_rows_that_fit() {
    let mut harness = page(with_long_line());
    assert_eq!(harness.query_all_by_label_contains("xxxx").count(), 1);

    harness.get_by_label("Wrap lines").click();
    harness.run();

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

    harness.get_by_label("Collect another report").click();
    harness.run();

    assert!(harness.query_by_label("Collect Full report").is_some());
}
