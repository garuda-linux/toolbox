use egui::accesskit::Role;
use egui::{Id, Key, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotResults};

use super::super::{ButtonKind, Confirmation, DialogWidth, ModalWrapper, confirm_buttons};
use crate::testing::{filled_rect, harness, painted_text};

#[derive(Debug, Default)]
struct Dialog {
    open: bool,
    answer: Option<Confirmation>,
    width: f32,
}

fn question(width: f32, height: f32) -> Harness<'static, Dialog> {
    let state = Dialog {
        open: true,
        ..Dialog::default()
    };

    let mut harness = harness(width, height).build_ui_state(
        |ui, dialog: &mut Dialog| {
            if !dialog.open {
                return;
            }

            let modal = ModalWrapper::new(Id::unique("question"), "Remove all changes")
                .title_bar("Close")
                .width(DialogWidth::Small);

            let outcome = modal.show(ui.ctx(), |ui| {
                ui.label("Remove all 3 queued changes?");
                let answer = confirm_buttons(ui, "Keep", "Remove all", ButtonKind::Danger);
                (answer, ui.min_rect().width())
            });

            dialog.width = outcome.inner.1;
            if let Some(answer) = outcome.inner.0 {
                dialog.answer = Some(answer);
                dialog.open = false;
            }

            if outcome.closed {
                dialog.open = false;
            }
        },
        state,
    );

    harness.run();
    harness
}

#[test]
fn the_dialog_is_named_for_screen_readers() {
    let harness = question(800.0, 600.0);

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Remove all changes")
            .is_some()
    );
}

#[test]
fn the_danger_button_confirms() {
    let mut harness = question(800.0, 600.0);

    harness
        .get_by_role_and_label(Role::Button, "Remove all")
        .click();
    harness.run();

    assert_eq!(harness.state().answer, Some(Confirmation::Confirmed));
}

#[test]
fn cancel_comes_first_in_the_tab_order() {
    let mut harness = question(800.0, 600.0);
    let buttons: Vec<String> = harness
        .query_all_by_role(Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .collect();

    let keep = buttons.iter().position(|label| label == "Keep");
    let remove = buttons.iter().position(|label| label == "Remove all");
    assert!(keep < remove, "{buttons:?}");

    harness.get_by_role_and_label(Role::Button, "Keep").click();
    harness.run();
    assert_eq!(harness.state().answer, Some(Confirmation::Cancelled));
}

#[test]
fn close_and_escape_close_the_dialog() {
    let mut harness = question(800.0, 600.0);
    harness.get_by_role_and_label(Role::Button, "Close").click();
    harness.run();
    assert!(!harness.state().open);

    let mut escaped = question(800.0, 600.0);
    escaped.key_press(Key::Escape);
    escaped.run();
    assert!(!escaped.state().open);
    assert_eq!(escaped.state().answer, None);
}

#[test]
fn a_dialog_never_grows_past_the_window() {
    let harness = question(300.0, 400.0);

    assert!(
        harness.state().width <= 300.0 - 40.0,
        "{}",
        harness.state().width
    );
}

#[test]
fn the_question_in_both_themes() {
    let mut results = SnapshotResults::new();
    for (theme, name) in [
        (Theme::Dark, "widgets/modal_dark"),
        (Theme::Light, "widgets/modal_light"),
    ] {
        let mut harness = question(640.0, 320.0);
        harness.ctx.set_theme(theme);
        harness.run();
        results.add(harness.try_snapshot(name));
    }
}

/// How opaque a dialog's text is on each of its first frames. The dialog
/// opens after the build, which runs every animation to its end.
fn opacities(instant: bool) -> Vec<u8> {
    let mut harness = harness(600.0, 400.0)
        .with_step_dt(1.0 / 60.0)
        .build_ui_state(
            move |ui, open: &mut bool| {
                if !*open {
                    return;
                }

                let mut modal = ModalWrapper::new(Id::unique("palette"), "Search");
                if instant {
                    modal = modal.instant();
                }

                modal.show(ui.ctx(), |ui| ui.label("Results"));
            },
            false,
        );

    // The harness turns egui's own motion off, this test watches it.
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = crate::theme::ANIMATION_TIME);

    *harness.state_mut() = true;
    (0..4)
        .filter_map(|_| {
            harness.step();
            painted_text(&harness, "Results").map(|text| text.fallback_color.a())
        })
        .collect()
}

#[test]
fn a_dialog_fades_in_unless_it_is_instant() {
    let faded = opacities(false);
    let instant = opacities(true);

    let full = instant.first().copied().unwrap_or_default();
    assert!(faded.iter().any(|alpha| *alpha < full), "fades: {faded:?}");
    assert!(
        instant.iter().all(|alpha| Some(alpha) == instant.first()),
        "instant: {instant:?}"
    );
}

#[test]
fn a_dialog_keeps_the_card_padding_around_its_content() {
    let harness = question(800.0, 600.0);
    let padding = crate::tokens::current(&harness.ctx).sizes.card_padding;

    let fill = crate::tokens::current(&harness.ctx).raised;
    let dialog = filled_rect(&harness, fill).expect("the dialog paints its frame");
    let title = harness
        .get_by_role_and_label(Role::Heading, "Remove all changes")
        .rect();

    let keep = harness.get_by_role_and_label(Role::Button, "Keep").rect();

    assert!(
        title.left() - dialog.left() >= f32::from(padding.left) - 0.5,
        "{dialog:?} {title:?}"
    );
    assert!(
        title.top() - dialog.top() >= f32::from(padding.top) - 0.5,
        "{dialog:?} {title:?}"
    );
    assert!(
        dialog.bottom() - keep.bottom() >= f32::from(padding.bottom) - 0.5,
        "{dialog:?} {keep:?}"
    );
}

#[test]
fn the_width_tokens_set_the_content_width() {
    let sizes = crate::tokens::Sizes::DEFAULT;
    let mut widths = Vec::new();
    for width in [DialogWidth::Small, DialogWidth::Medium, DialogWidth::Wide] {
        let mut harness = harness(1000.0, 600.0).build_ui_state(
            move |ui, seen: &mut f32| {
                let outcome = ModalWrapper::new(Id::unique("sized"), "Sized")
                    .width(width)
                    .show(ui.ctx(), |ui| ui.available_width());

                *seen = outcome.inner;
            },
            0.0,
        );

        harness.run();
        widths.push(*harness.state());
    }

    assert_eq!(
        widths,
        [sizes.dialog_small, sizes.dialog_medium, sizes.dialog_wide]
    );
}

#[test]
fn the_whole_window_is_blurred_under_a_dialog() {
    let harness = question(500.0, 400.0);
    let window = harness.ctx.content_rect();

    assert!(
        crate::testing::blurred_rects(&harness).contains(&window),
        "no blur covers the window"
    );
}
