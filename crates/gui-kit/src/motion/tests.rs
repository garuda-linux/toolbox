use std::time::Duration;

use egui::{Context, Id, RawInput, Ui, ViewportId};

use super::{
    SHEET, Timing, duration, fresh_presence, keyboard_marked, mark_keyboard, presence, settle,
    slide,
};
use crate::theme;

const STEP: f64 = 1.0 / 60.0;
const TIMING: Timing = Timing::new(0.18, 0.14);

fn frame<R>(ctx: &Context, time: f64, mut body: impl FnMut(&mut Ui) -> R) -> (R, Duration) {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::as_conversions,
        reason = "a frame's length fits a float"
    )]
    let input = RawInput {
        time: Some(time),
        predicted_dt: STEP as f32,
        ..RawInput::default()
    };

    let mut result = None;
    let mut output = ctx.run_ui(input, |ui| result = Some(body(ui)));
    output.textures_delta.clear();
    let result = result.expect("one pass");
    let delay = output
        .viewport_output
        .get(&ViewportId::ROOT)
        .map_or(Duration::MAX, |viewport| viewport.repaint_delay);

    (result, delay)
}

fn run(ctx: &Context, start: f64, seconds: f64, shown: bool) -> (f32, Duration) {
    let id = Id::unique("thing");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions,
        reason = "a few frames"
    )]
    let frames = (seconds / STEP).round() as u32;
    let mut last = (0.0, Duration::MAX);
    for index in 0..=frames {
        let time = f64::from(index).mul_add(STEP, start);
        last = frame(ctx, time, |ui| presence(ui.ctx(), id, shown, TIMING));
    }

    last
}

#[test]
fn what_was_there_before_shows_at_once() {
    let ctx = context();

    let (shown, delay) = frame(&ctx, 0.0, |ui| {
        presence(ui.ctx(), Id::unique("thing"), true, TIMING)
    });

    assert!((shown - 1.0).abs() < f32::EPSILON);
    assert_eq!(delay, Duration::MAX);
}

#[test]
fn showing_takes_its_time_and_ends_shown() {
    let ctx = context();
    let _: (f32, Duration) = run(&ctx, 0.0, 0.0, false);

    let (early, delay) = run(&ctx, STEP, STEP, true);
    assert!(early > 0.0 && early < 1.0, "{early}");
    assert_eq!(delay, Duration::ZERO, "frames while it moves");

    let (late, _) = run(&ctx, 3.0 * STEP, 0.3, true);
    assert!((late - 1.0).abs() < f32::EPSILON, "{late}");
}

#[test]
fn hiding_ends_hidden() {
    let ctx = context();
    let _: (f32, Duration) = run(&ctx, 0.0, 0.0, true);

    let (hidden, delay) = run(&ctx, STEP, 0.3, false);

    assert!(hidden.abs() < f32::EPSILON, "{hidden}");
    assert_eq!(delay, Duration::MAX, "no frames once it rests");
}

#[test]
fn with_less_motion_it_jumps_and_asks_for_no_frames() {
    let ctx = context();
    theme::set_reduced_motion(&ctx, true);
    let _: (f32, Duration) = run(&ctx, 0.0, 0.0, false);

    let (shown, delay) = run(&ctx, STEP, 0.0, true);

    assert!((shown - 1.0).abs() < f32::EPSILON, "{shown}");
    assert_eq!(delay, Duration::MAX);
    assert!(duration(&ctx, 0.18).abs() < f32::EPSILON);
}

#[test]
fn something_new_appears_rather_than_being_there() {
    let ctx = context();
    let id = Id::unique("note");

    let (first, _) = frame(&ctx, 0.0, |ui| fresh_presence(ui.ctx(), id, true, TIMING));
    assert!(first < 0.5, "{first}");

    let (second, _) = frame(&ctx, 0.5, |ui| fresh_presence(ui.ctx(), id, true, TIMING));
    assert!((second - 1.0).abs() < f32::EPSILON, "{second}");
}

#[test]
fn settle_puts_an_animation_at_its_end() {
    let ctx = context();
    let id = Id::unique("thing");
    let _: (f32, Duration) = run(&ctx, 0.0, 0.0, true);

    let (hidden, delay) = frame(&ctx, STEP, |ui| {
        settle(ui.ctx(), id, false);
        presence(ui.ctx(), id, false, TIMING)
    });

    assert!(hidden.abs() < f32::EPSILON, "{hidden}");
    assert_eq!(delay, Duration::MAX);
}

#[test]
fn a_keyboard_mark_holds_inside_the_marked_ui_for_a_moment() {
    let ctx = context();
    let row = Id::unique("row");
    let inside = |ui: &mut Ui| {
        let mut marked = (false, false);
        ui.scope_builder(egui::UiBuilder::new().scope_id(row), |ui| {
            ui.horizontal(|ui| marked.0 = keyboard_marked(ui));
        });
        marked.1 = keyboard_marked(ui);
        marked
    };

    let ((), _) = frame(&ctx, 0.0, |ui| mark_keyboard(ui.ctx(), row));

    let ((within, beside), _) = frame(&ctx, 0.5, inside);
    assert!(within);
    assert!(!beside, "only the marked ui");

    let ((later, _), _) = frame(&ctx, 2.0, inside);
    assert!(!later, "the mark wears off");
}

/// A context past the first frames, which ask for more frames anyway.
fn context() -> Context {
    let ctx = Context::default();
    for step in 1..=3_u8 {
        let ((), _) = frame(&ctx, f64::from(step) - 4.0, |_| ());
    }

    ctx
}

fn slide_for(ctx: &Context, start: f64, seconds: f64, shown: bool) -> f32 {
    let id = Id::unique("sheet");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions,
        reason = "a few frames"
    )]
    let frames = (seconds / STEP).round() as u32;
    let mut last = 0.0;
    for index in 0..=frames {
        let time = f64::from(index).mul_add(STEP, start);
        (last, _) = frame(ctx, time, |ui| slide(ui.ctx(), id, shown, SHEET));
    }

    last
}

#[test]
fn a_slide_out_starts_slow_and_ends_hidden() {
    let ctx = context();
    slide_for(&ctx, 0.0, 0.0, true);

    let half = f64::from(SHEET.exit) / 2.0_f64;
    let middle = slide_for(&ctx, STEP, half, false);
    assert!(middle > 0.7, "eases in, not straight: {middle}");

    let hidden = slide_for(&ctx, STEP + half + STEP, 0.3, false);
    assert!(hidden.abs() < f32::EPSILON, "{hidden}");
}

#[test]
fn a_slide_in_starts_fast_and_ends_shown() {
    let ctx = context();
    slide_for(&ctx, 0.0, 0.0, false);

    let half = f64::from(SHEET.enter) / 2.0_f64;
    let middle = slide_for(&ctx, STEP, half, true);
    assert!(middle > 0.7, "eases out: {middle}");

    let shown = slide_for(&ctx, STEP + half + STEP, 0.3, true);
    assert!((shown - 1.0).abs() < f32::EPSILON, "{shown}");
}

#[test]
fn with_less_motion_a_slide_jumps() {
    let ctx = context();
    theme::set_reduced_motion(&ctx, true);
    slide_for(&ctx, 0.0, 0.0, true);

    let hidden = slide_for(&ctx, STEP, 0.0, false);

    assert!(hidden.abs() < f32::EPSILON, "{hidden}");
}
