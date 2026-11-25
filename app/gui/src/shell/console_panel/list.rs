//! The step list of the Console sheet: one row per step with its state
//! and its command.

use egui::{Color32, Context, Frame, Label, ScrollArea, Ui, vec2};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::theme::show_scroll;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{card, line, rich};
use toolbox_plan::protocol::Failure;

use super::{STATE_SIZE, STEP_ROW_HEIGHT, STEPS_HEIGHT, follow_running};
use crate::console::{StepRow, StepState};
use crate::i18n;
use crate::i18n::Arg;

pub(super) fn steps(ui: &mut Ui, steps: &[StepRow], dry_run: bool) {
    if steps.is_empty() {
        return;
    }

    let tokens = tokens::current(ui.ctx());

    // As wide as the longest outcome in the list, so the commands line up
    // without a wide gap after a short "Done".
    let column = steps
        .iter()
        .map(|step| {
            let (words, _) = step_words(ui.ctx(), step.state, dry_run);
            line(
                ui,
                &words,
                Weight::SemiBold,
                STATE_SIZE,
                Color32::PLACEHOLDER,
            )
            .size()
            .x
        })
        .fold(0.0, f32::max)
        + tokens.sizes.control_gap;

    card(ui, |ui| {
        Frame::new()
            .inner_margin(tokens.sizes.steps_padding)
            .show(ui, |ui| {
                ui.scope(|ui| {
                    show_scroll(
                        ui,
                        ScrollArea::vertical()
                            .max_height(STEPS_HEIGHT)
                            .auto_shrink([false, true]),
                        |ui| {
                            ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
                            for step in steps {
                                let row = ui.horizontal(|ui| {
                                    ui.allocate_ui(vec2(column, STEP_ROW_HEIGHT), |ui| {
                                        ui.set_min_width(column);
                                        step_state(ui, step.state, dry_run);
                                    });
                                    let command = rich(
                                        ui,
                                        &step.command,
                                        Weight::Regular,
                                        SMALL,
                                        tokens.text,
                                    )
                                    .family(egui::FontFamily::Monospace);

                                    ui.add(Label::new(command).truncate());
                                });

                                follow_running(ui, step, &row.response);
                            }
                        },
                    );
                });
            });
    });
    ui.add_space(tokens.sizes.heading_gap);
}

pub(super) fn step_state(ui: &mut Ui, state: StepState, dry_run: bool) {
    let (words, color) = step_words(ui.ctx(), state, dry_run);
    ui.label(rich(ui, &words, Weight::SemiBold, STATE_SIZE, color));
}

fn step_words(ctx: &Context, state: StepState, dry_run: bool) -> (String, Color32) {
    let tokens = tokens::current(ctx);
    let text = |id: &str| i18n::text(ctx, id).into_owned();
    match state {
        StepState::Waiting => (text("step-waiting"), tokens.secondary),
        StepState::Running => (text("step-running"), tokens.accent),
        StepState::Succeeded => (text("step-succeeded"), tokens.success),
        // A real run skips a step that found nothing to do, a test skips
        // every step.
        StepState::Skipped if dry_run => (text("step-not-run"), tokens.secondary),
        StepState::Skipped => (text("step-nothing-to-do"), tokens.secondary),
        StepState::Cancelled => (text("step-cancelled"), tokens.warning),
        StepState::Failed(Failure::ExitCode { code }) => (
            i18n::text_with(ctx, "step-failed-exit", &[("code", Arg::from(code))]).into_owned(),
            tokens.error,
        ),
        StepState::Failed(Failure::Signal { signal }) => (
            i18n::text_with(ctx, "step-failed-signal", &[("signal", Arg::from(signal))])
                .into_owned(),
            tokens.error,
        ),
        StepState::Failed(Failure::Unknown) => (text("step-failed-unknown"), tokens.error),
    }
}
