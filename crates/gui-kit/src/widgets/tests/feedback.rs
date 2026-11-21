use std::time::Duration;

use egui::accesskit::Role;
use egui::{Theme, ViewportId};
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};

use super::super::loading::loading_dots;
use super::super::{
    ButtonKind, card, dimmed, empty_state, empty_state_action, loading_indicator, pill_button,
    skeleton_rows,
};
use crate::testing::{Cover, cover, harness, repaints_once_settled};
use crate::{UiExt as _, icons, theme};

fn repaint_delay<State>(harness: &Harness<'_, State>) -> Duration {
    harness
        .output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map_or(Duration::MAX, |viewport| viewport.repaint_delay)
}

#[test]
fn skeleton_rows_are_one_named_node_paced_below_the_display() {
    let mut harness = harness(600.0, 400.0).build_ui(|ui| {
        card(ui, |ui| skeleton_rows(ui, 5, 68.0, "Reading the services…"));
    });

    harness.run();

    assert_eq!(
        harness.query_all_by_label("Reading the services…").count(),
        1
    );
    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the services…")
            .is_some()
    );
    assert!(repaint_delay(&harness) > Duration::ZERO, "no busy loop");
}

#[test]
fn a_loading_indicator_is_a_progress_indicator_paced_below_the_display() {
    let mut harness = harness(400.0, 100.0).build_ui(|ui| {
        loading_indicator(ui, "Asking dkms for its modules…");
    });

    harness.run_steps(2);

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Asking dkms for its modules…")
            .is_some()
    );
    let delay = repaint_delay(&harness);
    assert!(
        delay >= Duration::from_millis(30) && delay <= Duration::from_millis(33),
        "{delay:?}"
    );
}

#[test]
fn with_less_motion_the_loading_indicator_stands_still() {
    let mut harness = harness(400.0, 100.0).build_ui(|ui| {
        ui.loading("Reading the system…");
    });

    theme::set_reduced_motion(&harness.ctx, true);

    harness.run_steps(2);

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the system…")
            .is_some()
    );
    assert_eq!(repaint_delay(&harness), Duration::MAX);
}

#[test]
fn an_unread_state_reports_its_button_centred_under_the_body() {
    let mut harness = harness(600.0, 300.0).build_ui_state(
        |ui, clicked: &mut bool| {
            let title = "The services could not be read";
            let body = "systemd is not reachable.";
            *clicked |=
                empty_state_action(ui, icons::DIALOG_ERROR, title, body, "Read again").clicked();
        },
        false,
    );

    harness.run();
    let body = harness.get_by_label("systemd is not reachable.").rect();
    let button = harness.get_by_role_and_label(Role::Button, "Read again");
    let rect = button.rect();
    assert!(rect.top() > body.bottom(), "{rect:?} under {body:?}");
    assert!((rect.center().x - body.center().x).abs() < 1.0, "{rect:?}");

    button.click();
    harness.run();
    assert!(*harness.state());
}

#[test]
fn a_dimmed_row_draws_at_reduced_opacity_only_while_busy() {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        |ui, seen: &mut (f32, f32)| {
            seen.0 = dimmed(ui, false, |ui| ui.opacity());
            seen.1 = dimmed(ui, true, |ui| ui.opacity());
        },
        (0.0, 0.0),
    );

    harness.run();

    let (resting, busy) = *harness.state();
    assert!((resting - 1.0).abs() < f32::EPSILON);
    assert!(busy < 0.7, "{busy}");
}

#[test]
fn feedback_in_both_themes() {
    let mut results = SnapshotResults::new();
    for (theme, name) in [
        (Theme::Dark, "widgets/feedback_dark"),
        (Theme::Light, "widgets/feedback_light"),
    ] {
        let mut harness = harness(640.0, 540.0).with_theme(theme).build_ui(|ui| {
            card(ui, |ui| skeleton_rows(ui, 2, 68.0, "Reading the services…"));
            ui.add_space(12.0);
            _ = empty_state_action(
                ui,
                icons::DIALOG_ERROR,
                "The services could not be read",
                "systemd is not reachable.",
                "Read again",
            );
            empty_state(
                ui,
                icons::SYSTEM_SEARCH,
                "Nothing matches the search",
                "No entry of this page has “pipe” in its name.",
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                pill_button(ui, "Cancel", ButtonKind::Soft);
                pill_button(ui, "Remove all", ButtonKind::Danger);
            });
        });

        theme::set_reduced_motion(&harness.ctx, true);

        harness.run();
        results.add(harness.try_snapshot(name));
    }
}

#[test]
fn loading_dots_beside_their_words_leave_the_words_one_node() {
    let mut harness = harness(400.0, 100.0).build_ui(|ui| {
        ui.horizontal(|ui| {
            loading_dots(ui);
            ui.label("Applying changes…");
        });
    });

    harness.run_steps(2);

    assert_eq!(harness.query_all_by_label("Applying changes…").count(), 1);
    assert!(
        harness.query_by_role(Role::ProgressIndicator).is_none(),
        "no unnamed progress node"
    );
}

#[test]
fn loading_dots_ask_for_frames_only_while_they_move() {
    for less_motion in [false, true] {
        let mut harness = harness(400.0, 100.0).build_ui(|ui| {
            loading_dots(ui);
        });

        theme::set_reduced_motion(&harness.ctx, less_motion);
        let repaints = repaints_once_settled(&mut harness);

        assert_eq!(repaints.is_empty(), less_motion, "{repaints:?}");
    }
}

#[test]
fn loading_dots_in_a_covered_window_ask_for_no_frames() {
    for how in [Cover::Minimized, Cover::Occluded] {
        let mut harness = harness(400.0, 100.0).build_ui(|ui| {
            loading_dots(ui);
        });

        cover(&mut harness, how);
        let repaints = repaints_once_settled(&mut harness);

        assert!(repaints.is_empty(), "{how:?}: {repaints:?}");
    }
}

#[test]
fn loading_dots_in_a_shown_window_without_the_focus_still_move() {
    let mut harness = harness(400.0, 100.0).build_ui(|ui| {
        loading_dots(ui);
    });

    harness.input_mut().focused = false;
    let repaints = repaints_once_settled(&mut harness);

    assert!(!repaints.is_empty());
}
