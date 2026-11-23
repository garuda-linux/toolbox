use egui_kittest::kittest::Queryable as _;
use toolbox_services::diagnostics::page::Page;

use super::{page, plain_report};

/// The folded rows' line above the report.
const COLLECT: &str = "Collect Another Report";

fn long_report() -> Page {
    let text = (0_u16..400).fold(String::new(), |mut text, line| {
        use std::fmt::Write as _;
        _ = writeln!(text, "kernel: line {line}");
        text
    });

    plain_report(text)
}

fn wheel_at(harness: &mut super::Shown, at: egui::Pos2) {
    harness.event(egui::Event::PointerMoved(at));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -300.0),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    harness.run_steps(10);
}

#[test]
fn only_the_report_scrolls_and_the_page_stays() {
    let mut harness = page(long_report());
    let collect = harness.get_by_label(COLLECT).rect();

    // Beside the folded rows, where only the page could take the wheel.
    wheel_at(
        &mut harness,
        egui::pos2(collect.right() + 200.0, collect.center().y),
    );
    assert!(
        (harness.get_by_label(COLLECT).rect().top() - collect.top()).abs() < 0.5,
        "the page scrolled under a report that fills it"
    );

    let first = harness.get_by_label("kernel: line 0").rect();
    wheel_at(&mut harness, first.center());
    assert!(
        harness
            .query_by_label("kernel: line 0")
            .is_none_or(|line| line.rect().top() < first.top()),
        "the report did not scroll"
    );
    assert!((harness.get_by_label(COLLECT).rect().top() - collect.top()).abs() < 0.5);
}

/// A page that fills the window keeps between its report and the bottom
/// edge the gap its cards keep between them.
#[test]
fn a_long_report_runs_down_to_a_card_gap_above_the_bottom() {
    let harness = page(long_report());
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);

    // The report's card is the largest card the page paints.
    let card = toolbox_gui_kit::testing::filled_rects(&harness, tokens.card)
        .into_iter()
        .max_by(|one, other| one.area().total_cmp(&other.area()))
        .expect("the report's card is painted");

    // kittest shows the page in a default central panel, whose margin the
    // app's window does not have.
    let panel = egui::Frame::central_panel(&harness.ctx.global_style()).inner_margin;
    let bottom = harness.ctx.content_rect().bottom() - f32::from(panel.bottom);
    let gap = tokens.sizes.card_gap;

    assert!(
        (bottom - card.bottom() - gap).abs() < 0.5,
        "the report's card ends {} above the bottom",
        bottom - card.bottom()
    );
}
