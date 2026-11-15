//! Closing the window while a plan runs would cancel it unseen, and
//! closing it during a Config Files review would drop the edit, so the
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
    asking: Option<Question>,
    /// The user chose to quit: the window closes once the cancelled plan
    /// ended.
    after_run: bool,
    /// The user chose to drop the edit, so the next close asks no more.
    edit_dropped: bool,
}

/// What would be lost by closing now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Question {
    Running,
    Edit,
}

/// The text ids of a question: title, detail, keep, quit.
struct Texts {
    question: &'static str,
    detail: &'static str,
    keep: &'static str,
    quit: &'static str,
}

impl Question {
    const fn texts(self) -> Texts {
        match self {
            Self::Running => Texts {
                question: "close-running-question",
                detail: "close-running-detail",
                keep: "close-running-keep",
                quit: "close-running-quit",
            },
            Self::Edit => Texts {
                question: "close-edit-question",
                detail: "close-edit-detail",
                keep: "close-edit-keep",
                quit: "close-edit-quit",
            },
        }
    }
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
        if !asked {
            return;
        }

        if let Some(question) = self.at_stake() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.closing.asking = Some(question);
        }
    }

    pub(super) fn close_after_run(&mut self, ctx: &Context) {
        if std::mem::take(&mut self.closing.after_run) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }

    pub(super) fn ask_before_close(&mut self, ctx: &Context) {
        if self.closing.asking.is_none() {
            return;
        }

        // The plan ended or the edit was applied while the question was
        // open. A run that ended may leave an edit to ask about instead.
        let Some(asking) = self.at_stake() else {
            self.closing.asking = None;
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        };

        self.closing.asking = Some(asking);

        let texts = asking.texts();
        let tokens = tokens::current(ctx);
        let question = i18n::text(ctx, texts.question);
        let close = i18n::text(ctx, "dialog-close");
        let modal = ModalWrapper::new(Id::unique("toolbox_gui::app::closing"), &question)
            .title_bar(&close)
            .show(ctx, |ui| {
                let detail = i18n::text(ctx, texts.detail);
                ui.add(
                    Label::new(rich(ui, &detail, Weight::Regular, BODY, tokens.secondary)).wrap(),
                );
                confirm_buttons(
                    ui,
                    &i18n::text(ctx, texts.keep),
                    &i18n::text(ctx, texts.quit),
                    ButtonKind::Danger,
                )
            });

        if modal.inner == Some(Confirmation::Confirmed) {
            match asking {
                Question::Running => {
                    self.closing.after_run = true;
                    self.to_service(ApplyCommand::Cancel);
                }
                Question::Edit => {
                    self.closing.edit_dropped = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }

        if modal.inner.is_some() || modal.closed {
            self.closing.asking = None;
        }
    }

    /// What closing the window now would lose. A run comes first: its end
    /// may still write the edit.
    fn at_stake(&self) -> Option<Question> {
        if self.console.running() {
            return Some(Question::Running);
        }

        (self.services.configs.unapplied() && !self.closing.edit_dropped).then_some(Question::Edit)
    }
}
