//! The viewer's header line names the report it holds, when it came in and
//! how many lines it has.

use egui_kittest::kittest::Queryable as _;
use toolbox_services::diagnostics::page::Page;

use super::{page, report};

#[test]
fn the_header_names_its_report_time_and_line_count_above_the_lines() {
    let harness = page(Page {
        collected: Some(1_792_327_265),
        ..report()
    });

    // The rows above name it too, the header comes last.
    let caption = harness
        .get_all_by_label("Kernel log")
        .last()
        .expect("the header names the report")
        .rect();

    let line = harness.get_by_label("kernel: hello").rect();

    assert!(
        caption.bottom() <= line.top(),
        "the caption {caption:?} stands above the report {line:?}"
    );
    assert!(
        harness
            .query_by_label("· collected 12:41 UTC · 2 lines")
            .is_some()
    );
}

#[test]
fn a_report_of_unknown_time_names_only_its_lines() {
    let harness = page(report());

    assert!(harness.query_by_label("· 2 lines").is_some());
}
