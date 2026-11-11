use egui_kittest::kittest::Queryable as _;
use toolbox_services::diagnostics::page::{Page, State};

use super::page;

fn long_report() -> Page {
    let text = (0_u16..400).fold(String::new(), |mut text, line| {
        use std::fmt::Write as _;
        _ = writeln!(text, "kernel: line {line}");
        text
    });

    Page {
        state: State::Report,
        selection: "dmesg".to_owned(),
        report: text.clone(),
        raw: text,
        ..Page::default()
    }
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
    let collect = harness.get_by_label("Collect").rect();

    // Beside the buttons, where only the page could take the wheel.
    wheel_at(
        &mut harness,
        egui::pos2(collect.right() + 200.0, collect.center().y),
    );
    assert!(
        (harness.get_by_label("Collect").rect().top() - collect.top()).abs() < 0.5,
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
    assert!((harness.get_by_label("Collect").rect().top() - collect.top()).abs() < 0.5);
}

/// A page that fills the window keeps between its report and the bottom
/// edge the gap its cards keep between them.
#[test]
fn a_long_report_runs_down_to_a_card_gap_above_the_bottom() {
    let harness = page(long_report());
    let report = harness.get_by_role(egui::accesskit::Role::Document).rect();

    // kittest shows the page in a default central panel, whose margin the
    // app's window does not have.
    let panel = egui::Frame::central_panel(&harness.ctx.global_style()).inner_margin;
    let bottom = harness.ctx.content_rect().bottom() - f32::from(panel.bottom);
    let gap = toolbox_gui_kit::tokens::current(&harness.ctx)
        .sizes
        .card_gap;

    assert!(
        (bottom - report.bottom() - gap).abs() < 0.5,
        "the report ends {} above the bottom",
        bottom - report.bottom()
    );
}
