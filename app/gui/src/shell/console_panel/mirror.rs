//! The offer under a failed install whose mirror lacked a package file:
//! one plan that updates the system and runs the same install again.

use egui::Ui;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{Tone, banner};

use crate::command::{CommandSender, ConsoleCommand};
use crate::console::{Console, Status};
use crate::i18n;

/// Only once the plan ended, as the offer starts a new one.
pub(super) fn offer(ui: &mut Ui, console: &Console, commands: &CommandSender) {
    if console.status() == Status::Running || console.mirror_retry().is_none() {
        return;
    }

    let ctx = ui.ctx().clone();
    let text = i18n::text(&ctx, "console-mirror-missing");
    let retry = i18n::text(&ctx, "console-update-and-retry");
    if banner(ui, Tone::Warning, &text, &[&retry]).is_some() {
        commands.send(ConsoleCommand::UpdateAndRetry);
    }

    ui.add_space(ui.tokens().sizes.control_gap);
}
