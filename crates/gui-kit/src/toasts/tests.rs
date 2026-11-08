mod layout;

use std::time::Duration;

use egui::ViewportId;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::{OFFSET, ToastKind, ToastTexts, Toasts, raise};
use crate::testing::harness;
use crate::tokens::Sizes;
use crate::widgets::{SearchTexts, page_header, page_toolbar, search_field};

/// Around a note's text, at least its frame's inner margin.
const FRAME_MARGIN: f32 = 14.0;

const TEXTS: ToastTexts<'static> = ToastTexts {
    name: "Notification",
    close: "Close",
};

fn shown(toasts: Toasts) -> Harness<'static, Toasts> {
    let mut harness = harness(800.0, 400.0).build_ui_state(
        |ui, toasts: &mut Toasts| {
            let page = ui.max_rect();
            toasts.show(ui.ctx(), page, TEXTS);
        },
        toasts,
    );

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
fn a_note_is_an_alert_with_its_text() {
    let harness = shown(with(ToastKind::Info, "The link did not open."));

    assert!(
        harness
            .query_by_role_and_label(Role::Alert, "Notification: The link did not open.")
            .is_some()
    );
    assert!(harness.query_by_label("The link did not open.").is_some());
}

#[test]
fn an_info_note_goes_after_four_seconds() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));

    // The first frames took a moment already.
    later(&mut harness, 2.0);
    assert_eq!(harness.state().texts(), ["Copied"]);

    later(&mut harness, 2.5);
    assert!(harness.state().is_empty());
}

#[test]
fn hovering_an_info_note_keeps_it() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));
    harness.get_by_label("Copied").hover();
    harness.step();

    later(&mut harness, 10.0);

    assert_eq!(harness.state().texts(), ["Copied"]);
}

#[test]
fn an_error_goes_after_ten_seconds() {
    let mut harness = shown(with(ToastKind::Error, "Firefox did not start."));

    later(&mut harness, 8.0);
    assert_eq!(harness.state().texts(), ["Firefox did not start."]);

    later(&mut harness, 2.5);
    assert!(harness.state().is_empty());
}

#[test]
fn an_error_can_be_closed_sooner() {
    let mut harness = shown(with(ToastKind::Error, "Firefox did not start."));

    harness.get_by_role_and_label(Role::Button, "Close").click();
    harness.step();
    later(&mut harness, 0.1);
    later(&mut harness, 0.1);
    assert!(harness.state().is_empty());
}

#[test]
fn an_info_note_has_no_close_button() {
    let harness = shown(with(ToastKind::Info, "Copied"));

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Close")
            .is_none()
    );
}

#[test]
fn notes_stack_downward_from_the_top_right() {
    let mut toasts = with(ToastKind::Error, "First");
    toasts.add(ToastKind::Error, "Second");
    let harness = shown(toasts);

    let first = harness.get_by_label("First").rect();
    let second = harness.get_by_label("Second").rect();

    assert!(second.min.y > first.max.y, "{first:?} {second:?}");
    assert!(first.min.x > 400.0, "at the right: {first:?}");
    assert!(first.min.y < 100.0, "{first:?}");
}

#[test]
fn a_widget_raises_a_note_through_the_context() {
    let mut harness = shown(Toasts::default());

    raise(&harness.ctx, ToastKind::Info, "Copied");
    harness.run_steps(2);

    assert_eq!(harness.state().texts(), ["Copied"]);
}

#[test]
fn notes_start_beside_the_title_even_with_a_header_and_toolbar() {
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
                |search, ui| _ = search_field(ui, search, texts, Sizes::DEFAULT.search_width),
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

    let top = harness
        .get_by_label("The services could not be read.")
        .rect()
        .min
        .y;

    let bare = shown(with(ToastKind::Error, "The services could not be read."));
    let bare_top = top_of(&bare, "The services could not be read.");

    assert!(
        (top - bare_top).abs() < 0.5,
        "the note starts at {top}, on a bare page at {bare_top}"
    );
    assert!(bare_top - FRAME_MARGIN < OFFSET.y * 3.0, "{bare_top}");
}

#[test]
fn without_a_page_header_notes_keep_to_the_page_top() {
    let harness = shown(with(ToastKind::Error, "First"));

    let note = harness.get_by_label("First").rect();

    assert!(note.min.y < 70.0, "{note:?}");
}

/// Notes drawn at sixty frames a second, so a motion takes frames.
fn smooth(toasts: Toasts) -> Harness<'static, Toasts> {
    harness(800.0, 400.0)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(60)
        .build_ui_state(
            |ui, toasts: &mut Toasts| {
                let page = ui.max_rect();
                toasts.show(ui.ctx(), page, TEXTS);
            },
            toasts,
        )
}

fn top_of(harness: &Harness<'static, Toasts>, text: &str) -> f32 {
    harness.get_by_label(text).rect().min.y
}

#[test]
fn a_new_note_drops_into_place() {
    let mut harness = smooth(Toasts::default());
    harness.run();
    harness.state_mut().add(ToastKind::Error, "First");
    harness.step();
    harness.step();
    let entering = top_of(&harness, "First");

    harness.run();
    let resting = top_of(&harness, "First");

    assert!(entering < resting, "{entering} above {resting}");
    assert!(resting - entering <= 8.0, "{entering} {resting}");
}

#[test]
fn a_closed_note_fades_before_it_goes() {
    let mut harness = smooth(with(ToastKind::Error, "First"));
    harness.run();

    harness.get_by_role_and_label(Role::Button, "Close").click();
    harness.step();
    harness.step();
    assert_eq!(harness.state().texts(), ["First"], "still fading");

    harness.run();
    assert!(harness.state().is_empty());
}

#[test]
fn the_notes_below_a_closed_one_move_up_rather_than_jump() {
    let mut toasts = with(ToastKind::Error, "First");
    toasts.add(ToastKind::Error, "Second");
    let mut harness = smooth(toasts);
    harness.run();
    let below = top_of(&harness, "Second");
    let first = top_of(&harness, "First");

    harness
        .get_all_by_role_and_label(Role::Button, "Close")
        .next()
        .unwrap()
        .click();
    while harness.state().texts().len() > 1 {
        harness.step();
    }

    harness.step();
    let moving = top_of(&harness, "Second");
    assert!(
        moving < below && moving > first,
        "{first} < {moving} < {below}"
    );

    harness.run();
    assert!((top_of(&harness, "Second") - first).abs() < 0.5);
}

#[test]
fn with_less_motion_a_note_shows_and_goes_at_once() {
    let mut harness = smooth(Toasts::default());
    crate::theme::set_reduced_motion(&harness.ctx, true);
    harness.run();
    harness.state_mut().add(ToastKind::Error, "First");
    harness.step();
    harness.step();
    let resting = top_of(&harness, "First");
    harness.step();

    assert!((top_of(&harness, "First") - resting).abs() < f32::EPSILON);

    // Only the countdown asks for a frame, no motion does.
    assert!(repaint_delay(&harness) > Duration::from_secs(1));

    harness.get_by_role_and_label(Role::Button, "Close").click();
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
fn an_info_note_waits_while_the_window_is_away() {
    let mut harness = shown(with(ToastKind::Info, "Copied"));
    harness.input_mut().focused = false;

    later(&mut harness, 10.0);
    assert_eq!(harness.state().texts(), ["Copied"]);

    harness.input_mut().focused = true;
    later(&mut harness, 4.5);
    assert!(harness.state().is_empty());
}
