use egui::Ui;
use toolbox_gui_kit::widgets::{ButtonKind, icon_button, pill_button, trailing};
use toolbox_gui_kit::{icons, tokens};

use super::{focus, output};
use crate::command::{AppCommand, CommandSender, UiCommand};
use crate::console::{Console, Status};
use crate::i18n;

/// The status left of them draws more or fewer widgets as the run goes,
/// which would renumber this group and drop its focus, so the group and
/// each control keep their own ids. Numbered by place, Close would take
/// over Cancel's id and the focus with it once Cancel goes.
pub(super) fn show(
    ui: &mut Ui,
    console: &Console,
    commands: &CommandSender,
    pill: impl FnOnce(&mut Ui),
) {
    let ctx = ui.ctx().clone();
    let gap = tokens::current(&ctx).sizes.card_gap;
    ui.push_id("console-header-controls", |ui| {
        trailing(ui, |ui| {
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
        });
    });
}
