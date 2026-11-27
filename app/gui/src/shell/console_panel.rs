mod controls;
pub mod focus;
mod idle;
mod list;
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
    Align, Context, CornerRadius, Frame, Id, Label, Layout, Rect, Response, Ui, UiBuilder, vec2,
};
use list::steps;
use toolbox_core::features::console::HelperFailure;
use toolbox_core::features::console::ending::Ending;
use toolbox_gui_kit::fonts::{SECTION, SMALL, Weight};
use toolbox_gui_kit::terminal_view::TerminalView;
use toolbox_gui_kit::theme::CARD_RADIUS;
use toolbox_gui_kit::widgets::{Tone, banner, push_height, rich, status_word};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_plan::StepId;
use toolbox_plan::protocol::Mode;
use toolbox_system::pkexec::Refusal;

use crate::apply::Problem;
use crate::command::{AppCommand, CommandSender, UiCommand};
use crate::console::{Console, Status, StepRow, StepState};
use crate::i18n;
use crate::i18n::Arg;

const STEPS_HEIGHT: f32 = tokens::Sizes::DEFAULT.steps_height;
const SHEET_RADIUS: u8 = toolbox_gui_kit::theme::POPOVER_RADIUS;
const STATE_SIZE: f32 = SMALL;
const STEP_ROW_HEIGHT: f32 = tokens::Sizes::DEFAULT.step_row_height;

#[derive(Clone, Copy, Debug)]
pub(super) struct Sheet {
    pub open: bool,
    /// The shortcut opened or closed it, so it jumps rather than slides.
    pub instant: bool,
    pub dry_run: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct At {
    pub sheet: Sheet,
    pub place: Place,
    /// A config file's editor runs. The page that tells how to go on lies
    /// under the sheet.
    pub editing: bool,
}

/// Draws the sheet at its place over the page. `pill` draws the queued
/// changes into the header, right to left beside Close. Returns the sheet's
/// rect while any of it shows, so the footer and the toasts keep clear.
pub fn show(
    ctx: &Context,
    at: At,
    console: &mut Console,
    commands: &CommandSender,
    pill: impl FnOnce(&mut Ui),
) -> Option<Rect> {
    let _ui = tracing::trace_span!("console ui").entered();
    let At {
        sheet,
        place,
        editing,
    } = at;

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
        notices(ui, console, editing);
        output::share(ui, console, commands);
        steps(ui, console.steps(), sheet.dry_run);
        mirror::offer(ui, console, commands);
        terminal(ui, console, commands);
    });

    if shown.dragged_shut {
        commands.send(UiCommand::ToggleConsole);
    }

    shown.rect
}

/// The ids are literals, so grep and the catalog checks see each one.
const fn refusal_id(refusal: Refusal) -> &'static str {
    match refusal {
        Refusal::Dismissed => "refusal-dismissed",
        Refusal::Denied => "refusal-denied",
        Refusal::NoAgent => "refusal-no-agent",
        Refusal::HelperMissing => "refusal-helper-missing",
        Refusal::Broken => "refusal-broken",
    }
}

const fn helper_id(kind: HelperFailure) -> &'static str {
    match kind {
        HelperFailure::MissingSecret => "helper-missing-secret",
        HelperFailure::Start => "helper-start",
        HelperFailure::Pkexec => "helper-pkexec",
        HelperFailure::NoPipes => "helper-no-pipes",
        HelperFailure::Lost => "helper-lost",
        HelperFailure::Ended => "helper-ended",
        HelperFailure::UserSteps => "helper-user-steps",
    }
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
                &[("count", Arg::from(skipped))],
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

    controls::review_hint(ui, console, commands);

    // The row is as tall as the push buttons it holds, so the title, the
    // status and the buttons share one middle.
    let row = vec2(ui.available_width(), push_height(ui));
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

        // Shown whole or not at all: a cut hint reads as a broken sentence.
        let hint = i18n::text(&ctx, "console-leave-hint");
        let words = rich(ui, &hint, Weight::Regular, SMALL, tokens.secondary);

        let body = egui::TextStyle::Body;
        let width = egui::WidgetText::from(words.clone()).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            body,
        );

        if width.size().x <= room.width() {
            let builder = UiBuilder::new()
                .max_rect(room)
                .layout(Layout::left_to_right(Align::Center));

            ui.scope_builder(builder, |ui| ui.add(Label::new(words)));
        }
    });
    ui.add_space(tokens.sizes.heading_gap);
}

fn notices(ui: &mut Ui, console: &Console, editing: bool) {
    let ctx = ui.ctx().clone();
    let mut notice = |tone: Tone, text: &str| {
        banner(ui, tone, text, &[]);
        ui.add_space(ui.tokens().sizes.control_gap);
    };

    if editing {
        notice(Tone::Neutral, &i18n::text(&ctx, "console-editing"));
    }

    if console.refused() {
        notice(Tone::Warning, &i18n::text(&ctx, "console-busy"));
    }

    if console.cancel_waits() {
        notice(Tone::Neutral, &i18n::text(&ctx, "console-cancel-waits"));
    }

    if let Some(reason) = console.rejected() {
        notice(
            Tone::Error,
            &i18n::text_with(&ctx, "console-rejected", &[("reason", Arg::from(reason))]),
        );
    }

    match console.problem() {
        Some(Problem::Refused(refusal)) => {
            notice(Tone::Error, &i18n::text(&ctx, refusal_id(*refusal)));
        }
        Some(Problem::HelperFailed { kind, detail }) => {
            notice(
                Tone::Error,
                &i18n::text_with(&ctx, helper_id(*kind), &[("detail", Arg::from(detail))]),
            );
        }
        None => {}
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

/// Brings the running step into view once as it starts, so a reader who
/// scrolls away keeps the place until the next step runs.
fn follow_running(ui: &Ui, step: &StepRow, row: &Response) {
    if step.state != StepState::Running {
        return;
    }

    let shown = Id::unique("console-running-step");
    if ui.data(|data| data.get_temp::<StepId>(shown)) != Some(step.id) {
        row.scroll_to_me(Some(Align::Center));
        ui.data_mut(|data| data.insert_temp(shown, step.id));
    }
}
