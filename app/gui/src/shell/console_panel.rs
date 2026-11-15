mod controls;
pub mod focus;
mod idle;
mod mirror;
mod output;
mod rim;
mod sheet;

#[cfg(any(test, feature = "testing"))]
pub use sheet::layer;
pub use sheet::{Place, cover, out};
#[cfg(test)]
mod tests;

use egui::{
    Align, Color32, Context, CornerRadius, Frame, Label, Layout, Rect, ScrollArea, Ui, UiBuilder,
    vec2,
};
use toolbox_core::features::console::ending::Ending;
use toolbox_gui_kit::fonts::{SECTION, SMALL, Weight};
use toolbox_gui_kit::terminal_view::TerminalView;
use toolbox_gui_kit::theme::{CARD_RADIUS, PILL_HEIGHT};
use toolbox_gui_kit::widgets::{Tone, banner, card, edge_ring, line, rich, status_word};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_plan::protocol::{Failure, Mode};

use crate::apply::Problem;
use crate::command::{AppCommand, CommandSender, UiCommand};
use crate::console::{Console, Status, StepRow, StepState};
use crate::i18n;

const STEPS_HEIGHT: f32 = tokens::Sizes::DEFAULT.steps_height;
const SHEET_RADIUS: u8 = toolbox_gui_kit::theme::POPOVER_RADIUS;
const STATE_SIZE: f32 = SMALL;
const STEP_ROW_HEIGHT: f32 = tokens::Sizes::DEFAULT.step_row_height;

#[derive(Clone, Copy, Debug)]
pub struct Sheet {
    pub open: bool,
    /// The shortcut opened or closed it, so it jumps rather than slides.
    pub instant: bool,
    pub dry_run: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct At {
    pub sheet: Sheet,
    pub place: Place,
}

/// Draws the sheet at its place over the page. `pill` draws the queued
/// changes into the header, right to left beside Close. Returns whether
/// any of the sheet shows, so the floating pill waits until it is gone.
pub fn show(
    ctx: &Context,
    at: At,
    console: &mut Console,
    commands: &CommandSender,
    pill: impl FnOnce(&mut Ui),
) -> bool {
    let _ui = tracing::trace_span!("console ui").entered();
    let At { sheet, place } = at;
    let tokens = tokens::current(ctx);
    let frame = Frame::new()
        .fill(tokens.pane)
        .corner_radius(CornerRadius {
            nw: SHEET_RADIUS,
            ne: SHEET_RADIUS,
            sw: 0,
            se: 0,
        })
        .inner_margin(tokens.sizes.sheet_padding);

    if sheet.open && sheet::escape_closes(ctx) {
        commands.send(UiCommand::ToggleConsole);
    }

    let grip = i18n::text(ctx, "console-resize");
    let shown = sheet::show(ctx, place, &grip, frame, |ui| {
        ui.name_pane(&i18n::text(ui.ctx(), "pane-console"));
        header(ui, console, sheet.dry_run, commands, pill);
        notices(ui, console);
        output::share(ui, console, commands);
        steps(ui, console.steps(), sheet.dry_run);
        mirror::offer(ui, console, commands);
        terminal(ui, console, commands);
    });

    if shown.dragged_shut {
        commands.send(UiCommand::ToggleConsole);
    }

    shown.rect.is_some()
}

fn ending_word(ctx: &Context, ending: Ending) -> (String, Tone) {
    match ending {
        Ending::Applied => (
            i18n::text(ctx, "console-status-succeeded").into_owned(),
            Tone::Success,
        ),
        Ending::Partly { skipped } => (
            i18n::text_with(
                ctx,
                "console-status-partly",
                &[("count", &skipped.to_string())],
            )
            .into_owned(),
            Tone::Success,
        ),
        Ending::Nothing => (
            i18n::text(ctx, "console-status-nothing").into_owned(),
            Tone::Success,
        ),
        Ending::Tested => (
            i18n::text(ctx, "console-status-tested").into_owned(),
            Tone::Neutral,
        ),
    }
}

fn header(
    ui: &mut Ui,
    console: &Console,
    dry_run: bool,
    commands: &CommandSender,
    pill: impl FnOnce(&mut Ui),
) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);

    // egui centres each widget on the row as tall as it is so far, so the
    // row starts as tall as the buttons that come last, or the status
    // centres on the title's shorter line.
    let row = vec2(ui.available_width(), PILL_HEIGHT);
    ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
        ui.label(rich(
            ui,
            &i18n::text(&ctx, "console-title"),
            Weight::Bold,
            SECTION,
            tokens.text,
        ));
        ui.add_space(tokens.sizes.caption_gap);
        match console.status() {
            Status::Idle => status_word(ui, &i18n::text(&ctx, "console-idle"), Tone::Neutral),
            Status::Running => status_word(
                ui,
                &i18n::text(&ctx, "console-status-running"),
                Tone::Accent,
            ),
            Status::Succeeded => {
                let (text, tone) = ending_word(&ctx, console.ending(Mode::of_dry_run(dry_run)));
                status_word(ui, &text, tone)
            }
            Status::Failed => {
                status_word(ui, &i18n::text(&ctx, "console-status-failed"), Tone::Error)
            }
        };

        // The hint takes the room between the state and the controls, cut
        // there, so a long state or pill never pushes the controls out.
        let state_end = ui.cursor().min.x + tokens.sizes.caption_gap;
        let controls = controls::show(ui, console, commands, pill);
        let room = Rect::from_x_y_ranges(
            state_end..=(controls - tokens.sizes.caption_gap).max(state_end),
            ui.min_rect().y_range(),
        );

        let hint = i18n::text(&ctx, "console-leave-hint");
        let builder = UiBuilder::new()
            .max_rect(room)
            .layout(Layout::left_to_right(Align::Center));

        ui.scope_builder(builder, |ui| {
            let words = rich(ui, &hint, Weight::Regular, SMALL, tokens.secondary);
            ui.add(Label::new(words).truncate());
        });
    });
    ui.add_space(tokens.sizes.heading_gap);
}

fn notices(ui: &mut Ui, console: &Console) {
    let ctx = ui.ctx().clone();
    let mut notice = |tone: Tone, text: &str| {
        banner(ui, tone, text, &[]);
        ui.add_space(ui.tokens().sizes.control_gap);
    };

    if console.refused() {
        notice(Tone::Warning, &i18n::text(&ctx, "console-busy"));
    }

    if console.cancel_waits() {
        notice(Tone::Neutral, &i18n::text(&ctx, "console-cancel-waits"));
    }

    if let Some(reason) = console.rejected() {
        notice(
            Tone::Error,
            &i18n::text_with(&ctx, "console-rejected", &[("reason", reason)]),
        );
    }

    match console.problem() {
        Some(Problem::Refused(refusal)) => {
            notice(
                Tone::Error,
                &i18n::text(&ctx, &format!("refusal-{}", refusal.code())),
            );
        }
        Some(Problem::HelperFailed { kind, detail }) => {
            let id = format!("helper-{}", kind.as_str());
            notice(
                Tone::Error,
                &i18n::text_with(&ctx, &id, &[("detail", detail)]),
            );
        }
        None => {}
    }
}

fn steps(ui: &mut Ui, steps: &[StepRow], dry_run: bool) {
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
                    toolbox_gui_kit::theme::show_scroll_bars(ui.style_mut());
                    ScrollArea::vertical()
                        .max_height(STEPS_HEIGHT)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
                            for step in steps {
                                ui.horizontal(|ui| {
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
                            }
                        });
                });
            });
    });
    ui.add_space(tokens.sizes.heading_gap);
}

fn step_state(ui: &mut Ui, state: StepState, dry_run: bool) {
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
            i18n::text_with(ctx, "step-failed-exit", &[("code", &code.to_string())]).into_owned(),
            tokens.error,
        ),
        StepState::Failed(Failure::Signal { signal }) => (
            i18n::text_with(
                ctx,
                "step-failed-signal",
                &[("signal", &signal.to_string())],
            )
            .into_owned(),
            tokens.error,
        ),
        StepState::Failed(Failure::Unknown) => (text("step-failed-unknown"), tokens.error),
    }
}

fn terminal(ui: &mut Ui, console: &mut Console, commands: &CommandSender) {
    let tokens = tokens::current(ui.ctx());
    let label = i18n::text(ui.ctx(), "console-terminal");
    let hint = i18n::text(ui.ctx(), "console-leave-hint");

    // egui puts its item spacing after the terminal too. Room left over
    // would let the resizable sheet grow by it on every frame.
    let below = ui.spacing().item_spacing.y;
    let room = (ui.available_size() - vec2(0.0, below)).max(egui::Vec2::ZERO);
    let idle = !console.has_output();
    let running = console.running();
    let fill = sheet::terminal_fill(ui.ctx());
    let output = ui
        .allocate_ui(room, |ui| {
            let framed = Frame::new()
                .fill(fill)
                .stroke(sheet::terminal_edge(ui.ctx()))
                .corner_radius(CornerRadius::same(CARD_RADIUS))
                .inner_margin(tokens.sizes.terminal_padding)
                .show(ui, |ui| {
                    TerminalView::new(console.terminal_mut(), &label)
                        .hint(&hint)
                        .ground(fill)
                        .running(running)
                        .show(ui)
                });

            // On the frame's edge, as the box fills its room and a ring
            // outside it would be cut.
            edge_ring(
                ui,
                &framed.inner.response,
                framed.response.rect,
                CARD_RADIUS,
            );
            framed.inner
        })
        .inner;

    // An empty terminal showed only its cursor, which says nothing about
    // what it is for.
    if idle {
        let ran = console.status() != Status::Idle;
        idle::idle_hint(ui, output.response.rect, ran);
    }

    let focused = output.response.has_focus();
    sheet::terminal_focused(ui.ctx(), focused);

    if !output.bytes.is_empty() {
        commands.send(AppCommand::ConsoleInput(output.bytes));
    }

    if let Some(size) = console.resize(output.fitting) {
        commands.send(AppCommand::ConsoleResize(size));
    }
}
