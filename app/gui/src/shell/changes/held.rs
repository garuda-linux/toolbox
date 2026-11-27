//! A held reason for a read that failed, such as a dismissed password
//! prompt, says so with Read again. Without it Apply would wait until the
//! user found the page that reads the part.

use egui::{Id, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{Tone, banner};

use super::Hold;
use crate::command::{Command, CommandSender, PackagesCommand, UsersCommand};
use crate::i18n;

/// The part whose read Read again started, so the banner stays while it
/// reads. Gone, the popover would shrink and jump for that time.
fn again_id() -> Id {
    Id::unique("toolbox_gui::changes::read_again")
}

/// The read that Read again starts for `reason`, none for a reason that
/// still waits on its own.
fn read_again(reason: &str) -> Option<Command> {
    match reason {
        "changes-unread-locks" => Some(UsersCommand::Refresh.into()),
        "changes-unread-aur" => Some(PackagesCommand::Refresh.into()),
        "changes-wait-orders" => Some(PackagesCommand::ReadOrders.into()),
        _ => None,
    }
}

fn part(reason: &str) -> Option<&'static str> {
    match reason {
        "changes-unread-locks" | "changes-wait-locks" => Some("locks"),
        "changes-unread-aur" | "changes-wait-aur" => Some("aur"),
        _ => None,
    }
}

pub fn unread(ui: &mut Ui, hold: &Hold, commands: &CommandSender) {
    let reason = hold.id;
    let Some(part) = part(reason) else {
        if let Some(again) = read_again(reason) {
            let button = i18n::text(ui.ctx(), "common-read-again");
            ui.add_space(ui.tokens().sizes.control_gap);
            if banner(ui, Tone::Warning, &hold.text, &[&*button])
                .value
                .is_some()
            {
                commands.send(again);
            }
        } else {
            forget(ui.ctx());
        }

        return;
    };

    let button = i18n::text(ui.ctx(), "common-read-again");
    if let Some(again) = read_again(reason) {
        let text = i18n::text(ui.ctx(), reason);
        ui.add_space(ui.tokens().sizes.control_gap);
        if banner(ui, Tone::Warning, &text, &[&*button])
            .value
            .is_some()
        {
            commands.send(again);
            ui.data_mut(|data| data.insert_temp(again_id(), part));
        }

        return;
    }

    if ui.data(|data| data.get_temp::<&'static str>(again_id())) != Some(part) {
        return;
    }

    // The text of the failed read, with the button resting, so nothing
    // changes size and the popover holds still.
    let failed = match part {
        "locks" => "changes-unread-locks",
        _ => "changes-unread-aur",
    };

    let text = i18n::text(ui.ctx(), failed);
    ui.add_space(ui.tokens().sizes.control_gap);
    ui.enabled_ui(false, |ui| banner(ui, Tone::Warning, &text, &[&*button]));
}

/// Nothing holds Apply any more, so a later wait is a first read again.
pub fn forget(ctx: &egui::Context) {
    ctx.data_mut(|data| data.remove_temp::<&'static str>(again_id()));
}

#[cfg(test)]
mod tests;
