//! Closing the window while a plan runs would cancel it unseen, so the
//! window asks first. The app shortcuts that could close it wait while a
//! program in the Console owns the keys.

use egui::{Context, Id, Label, ViewportCommand};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, Confirmation, ModalWrapper, confirm_buttons, rich};
use toolbox_gui_kit::{terminal_view, tokens};

use super::App;
use crate::apply::ApplyCommand;
use crate::command::UiCommand;
use crate::i18n;

#[derive(Debug, Default)]
pub struct Closing {
    asking: bool,
    /// The user chose to quit: the window closes once the cancelled plan
    /// ended.
    after_run: bool,
}

impl App {
    /// The shortcuts pressed this frame. A program in the Console gets
    /// every key, Ctrl+Tab leaves it.
    pub(super) fn pressed(ctx: &Context) -> Vec<UiCommand> {
        if terminal_view::owns_keys(ctx) {
            return Vec::new();
        }

        let text_focused = ctx.text_edit_focused();
        ctx.input_mut(|input| UiCommand::pressed(input, text_focused))
    }

    pub(super) fn guard_close(&mut self, ctx: &Context) {
        let asked = ctx.input(|input| input.viewport().close_requested());
        if asked && self.console.running() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.closing.asking = true;
        }
    }

    pub(super) fn close_after_run(&mut self, ctx: &Context) {
        if std::mem::take(&mut self.closing.after_run) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }

    pub(super) fn ask_before_close(&mut self, ctx: &Context) {
        if !self.closing.asking {
            return;
        }

        // The plan ended while the question was open.
        if !self.console.running() {
            self.closing.asking = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        let tokens = tokens::current(ctx);
        let question = i18n::text(ctx, "close-running-question");
        let close = i18n::text(ctx, "dialog-close");
        let modal = ModalWrapper::new(Id::unique("toolbox_gui::app::closing"), &question)
            .title_bar(&close)
            .show(ctx, |ui| {
                let detail = i18n::text(ctx, "close-running-detail");
                ui.add(
                    Label::new(rich(ui, &detail, Weight::Regular, BODY, tokens.secondary)).wrap(),
                );
                confirm_buttons(
                    ui,
                    &i18n::text(ctx, "close-running-keep"),
                    &i18n::text(ctx, "close-running-quit"),
                    ButtonKind::Danger,
                )
            });

        if modal.inner == Some(Confirmation::Confirmed) {
            self.closing.after_run = true;
            self.to_service(ApplyCommand::Cancel);
        }

        if modal.inner.is_some() || modal.closed {
            self.closing.asking = false;
        }
    }
}
