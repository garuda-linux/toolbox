//! What the question shows is kept until it is answered, so the text
//! that goes out is the text the user read, even while a plan writes on.

use std::sync::Arc;

use egui::{Id, Ui};
use toolbox_core::features::diagnostics::Redacted;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{menu_button, menu_item};

use crate::command::{CommandSender, ConsoleCommand};
use crate::console::{Console, Status};
use crate::i18n;
use crate::share::view::{self as share, Question, ShareView, Subject};

#[derive(Clone, Debug, Default)]
struct Asking {
    question: Option<Question>,
    /// Shared, as egui hands temp data out by clone every frame.
    shareable: Arc<Redacted>,
}

fn asking_id() -> Id {
    Id::unique("toolbox_gui::console::share")
}

/// The ⋯ menu, once a plan ran or a program wrote something. Copy and
/// Share need output, Clear also empties the steps of a plan that printed
/// nothing, as a test run's.
pub(super) fn menu(ui: &mut Ui, console: &Console, commands: &CommandSender) {
    let output = console.has_output();
    if !output && console.steps().is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    menu_button(ui, &text("console-output-menu"), |ui| {
        let copying = ui.add_enabled_ui(output, |ui| menu_item(ui, &text("console-copy-output")));

        if copying.inner.clicked() {
            share::copy(
                &ctx,
                commands,
                &console.output_text(),
                "console-output-copied",
            );
        }

        let sharing = ui.add_enabled_ui(output && !console.share().running(), |ui| {
            menu_item(ui, &text("console-share-output"))
        });

        if sharing.inner.clicked() {
            let asking = Asking {
                question: Some(Question::Upload),
                shareable: Arc::new(console.shareable_output()),
            };

            ctx.data_mut(|data| data.insert_temp(asking_id(), asking));
        }

        // A running plan's programs still write to the terminal.
        let clearing = ui.add_enabled_ui(console.status() != Status::Running, |ui| {
            menu_item(ui, &text("console-clear-output"))
        });

        if clearing.inner.clicked() {
            commands.send(ConsoleCommand::ClearOutput);
        }
    });
}

pub(super) fn share(ui: &mut Ui, console: &Console, commands: &CommandSender) {
    let ctx = ui.ctx().clone();
    let view = ShareView {
        upload: console.share(),
        subject: Subject::Output,
        commands,
        hide: ConsoleCommand::HideShare.into(),
    };

    let mut asking: Asking = ctx.data(|data| data.get_temp(asking_id()).unwrap_or_default());
    if share::row(ui, &view, &mut asking.question) {
        ui.add_space(ui.tokens().sizes.control_gap);
    }

    if share::ask(&ctx, &view, &asking.shareable, &mut asking.question) {
        commands.send(ConsoleCommand::Share(asking.shareable.text().to_owned()));
    }

    if asking.question.is_none() {
        asking.shareable = Arc::default();
    }

    ctx.data_mut(|data| data.insert_temp(asking_id(), asking));
}
