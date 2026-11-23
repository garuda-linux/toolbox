use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::{MetricKind, MetricText, metric};
use crate::testing::{harness, spoken_words, text_colour};
use crate::tokens;

#[test]
fn an_unknown_number_says_why_in_its_place_and_no_dash() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        metric(
            ui,
            MetricKind::Primary,
            MetricText {
                name: "Updates pending",
                value: None,
                detail: "Could not check",
            },
            true,
        );
    });

    harness.run();

    let tokens = tokens::current(&harness.ctx);

    assert!(harness.query_by_label("—").is_none(), "no dash");
    assert_eq!(
        text_colour(&harness, "Could not check"),
        Some(tokens.secondary)
    );
    assert!(
        harness
            .query_by_label("Updates pending: Could not check")
            .is_some()
    );

    let card = harness
        .get_by_label("Updates pending: Could not check")
        .rect();

    let fill = crate::testing::fill_under(&harness, card.left_top() + egui::vec2(6.0, 6.0));
    assert_ne!(
        fill,
        Some(tokens.accent_tint),
        "an unknown figure is not highlighted"
    );
}

#[test]
fn the_details_of_a_row_of_figures_share_one_baseline() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        ui.horizontal_top(|ui| {
            ui.allocate_ui(egui::vec2(280.0, 0.0), |ui| {
                metric(
                    ui,
                    MetricKind::Primary,
                    MetricText {
                        name: "Updates pending",
                        value: Some("2"),
                        detail: "16 days ago",
                    },
                    true,
                );
            });
            ui.allocate_ui(egui::vec2(140.0, 0.0), |ui| {
                metric(
                    ui,
                    MetricKind::Secondary,
                    MetricText {
                        name: "AUR updates",
                        value: Some("1"),
                        detail: "none new",
                    },
                    true,
                );
            });
        });
    });

    harness.run();

    let primary = harness.get_by_label("16 days ago").rect();
    let secondary = harness.get_by_label("none new").rect();
    assert!(
        (primary.bottom() - secondary.bottom()).abs() < 0.5,
        "{primary:?} {secondary:?}"
    );
}

#[test]
fn a_primary_and_a_secondary_figure_stand_equally_tall() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        ui.horizontal_top(|ui| {
            ui.allocate_ui(egui::vec2(280.0, 0.0), |ui| {
                metric(
                    ui,
                    MetricKind::Primary,
                    MetricText {
                        name: "Updates pending",
                        value: Some("2"),
                        detail: "16 days ago",
                    },
                    true,
                );
            });
            ui.allocate_ui(egui::vec2(140.0, 0.0), |ui| {
                metric(
                    ui,
                    MetricKind::Secondary,
                    MetricText {
                        name: "AUR updates",
                        value: Some("1"),
                        detail: "",
                    },
                    true,
                );
            });
        });
    });

    harness.run();

    let primary = harness.get_by_label("Updates pending: 2").rect();

    let secondary = harness.get_by_label("AUR updates: 1").rect();
    assert!(
        (primary.height() - secondary.height()).abs() < 0.5,
        "{primary:?} {secondary:?}"
    );
}

#[test]
fn an_unknown_number_leaves_no_empty_label() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        let text = MetricText {
            name: "Disk",
            value: None,
            detail: "Could not read",
        };

        metric(ui, MetricKind::Secondary, text, false);
    });

    harness.run();

    let empty = harness
        .query_all_by_role(Role::Label)
        .filter(|node| spoken_words(node).is_empty())
        .count();

    assert_eq!(empty, 0, "a label with no words");
}

#[test]
fn a_metric_that_leads_somewhere_lights_up_under_the_pointer() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        let text = MetricText {
            name: "Disk",
            value: Some("40 %"),
            detail: "of 512 GB",
        };

        metric(ui, MetricKind::Secondary, text, true);
    });

    harness.run();
    let card = harness.get_by_label("Disk: 40 %").rect();
    let inside = card.left_top() + egui::vec2(6.0, 6.0);
    let rest = crate::testing::fill_under(&harness, inside);

    harness.get_by_label("Disk: 40 %").hover();
    harness.run();
    let tokens = tokens::current(&harness.ctx);

    assert_ne!(rest, Some(tokens.button_hover));
    assert_eq!(
        crate::testing::fill_under(&harness, inside),
        Some(tokens.button_hover)
    );
}
