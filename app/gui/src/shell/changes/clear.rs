//! The question before every queued change is removed.

use egui::{Context, Id};
use toolbox_gui_kit::widgets::{ButtonKind, DialogWidth, Question, question};

use crate::command::{Command, CommandSender, UiCommand};
use crate::i18n;

/// The question for [`UiCommand::ClearChanges`], which the app asks before
/// it removes anything.
pub fn ask_to_clear(
    ctx: &Context,
    count: &str,
    confirming: &mut Option<UiCommand>,
    commands: &CommandSender,
) {
    let title = i18n::text(ctx, "command-clear-changes");
    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text_with(ctx, "changes-remove-all-question", &[("count", count)]);
    let cancel = i18n::text(ctx, "changes-keep");
    let confirm = i18n::text(ctx, "changes-remove-all");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = question(ctx, Id::unique("toolbox_gui::changes::clear"), texts);
    if asked.confirmed {
        commands.send(Command::Confirmed(UiCommand::ClearChanges));
    }

    if asked.done {
        *confirming = None;
    }
}
