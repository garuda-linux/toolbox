use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_services::diagnostics::page::Page;

use super::{page, report};

const NARROW: u16 = 640;

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

#[test]
fn the_header_controls_stay_inside_the_card_at_any_width() {
    let mut harness = page(Page {
        collected: Some(1_792_327_265),
        ..report()
    });

    // Every 16 points from the narrow width up, so each control in turn is
    // the first one that no longer fits beside the title.
    for width in (NARROW..=1024).step_by(16).map(f32::from) {
        harness.set_size(egui::vec2(width, 900.0));
        harness.run();

        let card = harness
            .get_by_role_and_label(Role::Document, "Diagnostics report")
            .rect();

        let window = harness.ctx.content_rect();
        assert!(card.right() <= window.right(), "{width}: the card {card:?}");

        for (role, name) in [
            (Role::TextInput, "Search the report"),
            (Role::Button, "Copy Report"),
            (Role::Button, "Share Report…"),
        ] {
            let control = harness.get_by_role_and_label(role, name).rect();
            assert!(
                card.contains_rect(control),
                "{width}: {name} at {control:?} leaves the card {card:?}"
            );
        }
    }
}
