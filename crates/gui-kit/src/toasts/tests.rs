mod layout;

use std::time::Duration;

use egui::accesskit::Role;
use egui::{Ui, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::{ToastKind, ToastTexts, Toasts, raise};
use crate::testing::harness;
use crate::tokens::Sizes;
use crate::widgets::{SearchTexts, page_header, page_toolbar, search_field};

const TEXTS: ToastTexts<'static> = ToastTexts {
    name: "Notification",
};

fn draw(ui: &mut Ui, toasts: &mut Toasts) {
    let page = ui.max_rect();
    toasts.show(ui.ctx(), page, TEXTS);
}

fn shown(toasts: Toasts) -> Harness<'static, Toasts> {
    let mut harness = harness(800.0, 400.0).build_ui_state(draw, toasts);

    harness.run_steps(2);
    harness
}

fn later(harness: &mut Harness<'static, Toasts>, seconds: f64) {
    let now = harness.ctx.input(|input| input.time);
    harness.input_mut().time = Some(now + seconds);
    harness.step();
}

fn with(kind: ToastKind, text: &str) -> Toasts {
    let mut toasts = Toasts::default();
    toasts.add(kind, text);
    toasts
}

#[test]
fn a_note_is_a_status_message_with_its_text_and_a_failure_an_alert() {
    let harness = shown(with(ToastKind::Info, "The link did not open."));
    assert!(
        harness
            .query_by_role_and_label(Role::Status, "Notification: The link did not open.")
            .is_some()
    );

    let error = shown(with(ToastKind::Error, "Firefox did not start."));
    assert!(
        error
            .query_by_role_and_label(Role::Alert, "Notification: Firefox did not start.")
            .is_some()
    );
}

#[test]
fn a_note_goes_after_seven_seconds_as_kirigami_times_it() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));

    later(&mut harness, 5.5);
    assert_eq!(harness.state().texts(), ["Copied"]);

    later(&mut harness, 1.5);
    assert!(harness.state().is_empty());
}

#[test]
fn an_error_has_no_longer_timeout_than_a_note() {
    let mut harness = shown(with(ToastKind::Error, "Firefox did not start."));

    later(&mut harness, 5.5);
    assert_eq!(harness.state().texts(), ["Firefox did not start."]);

    later(&mut harness, 1.5);
    assert!(harness.state().is_empty());
}

#[test]
fn a_note_raised_after_an_idle_window_is_still_there_on_its_first_frame() {
    let mut harness = shown(Toasts::default());

    harness.state_mut().add(ToastKind::Info, "Copied");
    later(&mut harness, 10.0);
    harness.step();

    assert_eq!(harness.state().texts(), ["Copied"]);
}

#[test]
fn hovering_a_note_keeps_it() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));
    harness.get_by_label("Copied").hover();
    harness.step();

    later(&mut harness, 10.0);

    assert_eq!(harness.state().texts(), ["Copied"]);
}

#[test]
fn a_click_closes_a_note_and_it_has_no_close_button() {
    let mut harness = shown(with(ToastKind::Error, "Firefox did not start."));
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Close")
            .is_none()
    );

    harness.get_by_label("Firefox did not start.").click();
    harness.step();
    later(&mut harness, 0.1);
    later(&mut harness, 0.1);
    assert!(harness.state().is_empty());
}

#[test]
fn notes_stack_up_from_the_bottom_centre_with_the_newest_on_top() {
    let mut toasts = with(ToastKind::Info, "First");
    toasts.add(ToastKind::Info, "Second");
    let harness = shown(toasts);

    let first = harness.get_by_label("First").rect();
    let second = harness.get_by_label("Second").rect();

    assert!(second.max.y < first.min.y, "{first:?} {second:?}");
    assert!(first.max.y > 360.0, "at the bottom: {first:?}");
    assert!((first.center().x - 400.0).abs() < 1.0, "{first:?}");
}

#[test]
fn a_widget_raises_a_note_through_the_context() {
    let mut harness = shown(Toasts::default());

    raise(&harness.ctx, ToastKind::Info, "Copied");
    harness.run_steps(2);

    assert_eq!(harness.state().texts(), ["Copied"]);
}

#[test]
fn notes_sit_at_the_bottom_of_the_page_under_a_header_and_toolbar_too() {
    let mut harness = harness(800.0, 400.0).build_ui_state(
        |ui, (toasts, search): &mut (Toasts, String)| {
            let page = ui.max_rect();
            let texts = SearchTexts {
                name: "Search services",
                hint: "Name or description",
                clear: "Clear the search",
            };

            page_header(ui, "Services", "What runs and what starts.", 0.0, |_| {});
            page_toolbar(
                ui,
                search,
                |search, ui| {
                    _ = search_field(
                        ui,
                        "test-search",
                        search,
                        texts,
                        Sizes::DEFAULT.search_width,
                    );
                },
                |_, _| {},
            );

            toasts.show(ui.ctx(), page, TEXTS);
        },
        (
            with(ToastKind::Error, "The services could not be read."),
            String::new(),
        ),
    );

    harness.run_steps(2);

    let bottom = harness
        .get_by_label("The services could not be read.")
        .rect()
        .max
        .y;

    assert!(bottom > 360.0, "the note ends at {bottom}");
}

/// Notes drawn at sixty frames a second, so a motion takes frames.
fn smooth(toasts: Toasts) -> Harness<'static, Toasts> {
    harness(800.0, 400.0)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(60)
        .build_ui_state(draw, toasts)
}

fn top_of(harness: &Harness<'static, Toasts>, text: &str) -> f32 {
    harness.get_by_label(text).rect().min.y
}

#[test]
fn a_new_note_drops_into_place() {
    let mut harness = smooth(Toasts::default());
    harness.run();
    harness.state_mut().add(ToastKind::Info, "First");
    harness.step();
    harness.step();
    let entering = top_of(&harness, "First");

    harness.run();
    let resting = top_of(&harness, "First");

    assert!(entering < resting, "{entering} above {resting}");
    assert!(resting - entering <= 8.0, "{entering} {resting}");
}

#[test]
fn a_clicked_note_fades_before_it_goes() {
    let mut harness = smooth(with(ToastKind::Info, "First"));
    harness.run();

    harness.get_by_label("First").click();
    harness.step();
    harness.step();
    assert_eq!(harness.state().texts(), ["First"], "still fading");

    harness.run();
    assert!(harness.state().is_empty());
}

#[test]
fn the_notes_above_a_closed_one_move_down_rather_than_jump() {
    let mut toasts = with(ToastKind::Info, "First");
    toasts.add(ToastKind::Info, "Second");
    let mut harness = smooth(toasts);
    harness.run();
    let above = top_of(&harness, "Second");
    let first = top_of(&harness, "First");

    harness.get_by_label("First").click();
    while harness.state().texts().len() > 1 {
        harness.step();
    }

    harness.step();
    let moving = top_of(&harness, "Second");
    assert!(
        above < moving && moving < first,
        "{above} < {moving} < {first}"
    );

    harness.run();
    assert!((top_of(&harness, "Second") - first).abs() < 0.5);
}

#[test]
fn with_less_motion_a_note_shows_and_goes_at_once() {
    let mut harness = smooth(Toasts::default());
    crate::theme::set_reduced_motion(&harness.ctx, true);
    harness.run();
    harness.state_mut().add(ToastKind::Info, "First");
    harness.step();
    harness.step();
    let resting = top_of(&harness, "First");
    harness.step();

    assert!((top_of(&harness, "First") - resting).abs() < f32::EPSILON);

    // Only the countdown asks for a frame, no motion does.
    assert!(repaint_delay(&harness) > Duration::from_secs(1));

    harness.get_by_label("First").click();
    harness.step();
    assert!(harness.state().is_empty());
}

fn repaint_delay(harness: &Harness<'static, Toasts>) -> Duration {
    harness
        .output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map_or(Duration::MAX, |viewport| viewport.repaint_delay)
}

#[test]
fn a_note_waits_while_the_window_is_away() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));
    harness.input_mut().focused = false;

    later(&mut harness, 10.0);
    assert_eq!(harness.state().texts(), ["Copied"]);

    harness.input_mut().focused = true;
    later(&mut harness, 7.5);
    assert!(harness.state().is_empty());
}

#[test]
fn a_note_still_shows_on_the_frame_the_window_comes_back() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));
    harness.input_mut().focused = false;
    harness.step();

    harness.input_mut().focused = true;
    later(&mut harness, 10.0);
    assert_eq!(harness.state().texts(), ["Copied"]);

    later(&mut harness, 7.5);
    assert!(harness.state().is_empty());
}

#[test]
fn at_most_four_notes_stay_open_and_a_fifth_takes_the_place_of_the_oldest() {
    let mut toasts = Toasts::default();
    for text in ["one", "two", "three", "four", "five"] {
        toasts.add(ToastKind::Info, text);
    }

    let open: Vec<&str> = toasts
        .shown
        .iter()
        .filter(|toast| !toast.leaving())
        .map(|toast| toast.text.as_str())
        .collect();

    assert_eq!(open, ["two", "three", "four", "five"]);
}
