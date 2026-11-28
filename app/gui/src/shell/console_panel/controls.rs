use egui::Ui;
use toolbox_gui_kit::widgets::{
    ButtonKind, icon_button, pill_button, pill_width, split_row, trailing,
};
use toolbox_gui_kit::{icons, tokens};

use super::{focus, output};
use crate::command::{AppCommand, CommandSender, UiCommand};
use crate::console::{Console, Status};
use crate::i18n;

/// The line above the header while a review waits for Continue: the hint and
/// Continue. The header has no room for either beside its own buttons.
pub(super) fn review_hint(ui: &mut Ui, console: &Console, commands: &CommandSender) {
    if !console.review_waits() {
        return;
    }

    let ctx = ui.ctx().clone();
    let label = i18n::text(&ctx, "console-continue");

    // The button keeps its width and the hint wraps in the rest, so the
    // sheet never grows wider than the window.
    let ((), proceed) = split_row(
        ui,
        pill_width(ui, &label),
        |ui| {
            let hint = i18n::text(&ctx, "console-review-hint");
            ui.add(egui::Label::new(hint.as_ref()).wrap());
        },
        // Never the default: Enter reaches the review as the terminal's key,
        // and only a click on this button sends `y`.
        |ui| {
            ui.push_id("console-continue", |ui| {
                pill_button(ui, &label, ButtonKind::Primary)
            })
            .inner
        },
    );

    if proceed.clicked() {
        commands.send(AppCommand::ConsoleInput(b"y\r".to_vec()));
    }
}

/// The status left of them draws more or fewer widgets as the run goes,
/// which would renumber this group and drop its focus, so the group and
/// each control keep their own ids. Numbered by place, Close would take
/// over Cancel's id and the focus with it once Cancel goes.
///
/// Returns where the controls start, the room the leave hint may take.
pub(super) fn show(
    ui: &mut Ui,
    console: &Console,
    commands: &CommandSender,
    pill: impl FnOnce(&mut Ui),
) -> f32 {
    let ctx = ui.ctx().clone();
    let gap = tokens::current(&ctx).sizes.card_gap;
    ui.push_id("console-header-controls", |ui| {
        trailing(ui, |ui| {
            let left = ui.cursor().min.x;
            pill(ui);
            ui.add_space(gap);

            let running = console.status() == Status::Running;
            let cancel = running.then(|| {
                let shown = ui
                    .push_id("console-cancel", |ui| {
                        pill_button(ui, &i18n::text(&ctx, "console-cancel"), ButtonKind::Soft)
                    })
                    .inner;

                focus::take(&ctx, &shown);
                if shown.clicked() {
                    commands.send(AppCommand::CancelPlan);
                }

                shown
            });

            ui.push_id("console-menu", |ui| output::menu(ui, console, commands));

            let close = ui
                .push_id("console-close", |ui| {
                    icon_button(ui, icons::WINDOW_CLOSE, &i18n::text(&ctx, "console-hide"))
                })
                .inner;

            focus::take(&ctx, &close);
            focus::keep_in_header(&ctx, running, cancel.as_ref(), &close);
            if close.clicked() {
                commands.send(UiCommand::ToggleConsole);
            }

            left
        })
    })
    .inner
}
