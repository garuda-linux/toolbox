//! The links hold the key to the paste, so they are only shown and copied
//! when asked, never logged.

use egui::{Id, Label, Ui};
use toolbox_core::features::diagnostics::{Redacted, UploadFailure};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, Tone, banner, confirm_buttons,
    loading_label, menu_button, menu_item, pill_button, push_height, rich,
};
use toolbox_privatebin::{Paste, SERVER};
use toolbox_services::share::{Upload, UploadProblem};

use super::preview;
use crate::command::{AppCommand, Command, CommandSender};
use crate::i18n::Arg;
use crate::{i18n, links};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    Report,
    Output,
}

/// A question asked before the text goes out or the paste goes away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Question {
    Upload,
    /// Opening the delete link removes the paste for everyone.
    Delete,
}

/// One upload as its view needs it.
pub struct ShareView<'a> {
    pub upload: &'a Upload,
    pub subject: Subject,
    pub commands: &'a CommandSender,
    /// Sent by Hide and by the failure's Dismiss.
    pub hide: Command,
}

impl Subject {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Report => "report",
            Self::Output => "output",
        }
    }
}

impl ShareView<'_> {
    pub(super) fn text(&self, ctx: &egui::Context, id: &str) -> String {
        i18n::text_with(ctx, id, &[("subject", Arg::from(self.subject.as_str()))]).into_owned()
    }
}

/// The link in one row: Copy link, Hide, then the other link actions
/// (open, the delete link) in a ⋯ menu at the end, as a row's menu is.
///
/// Without a paste, the running upload or its failure is shown instead.
pub fn row(ui: &mut Ui, view: &ShareView<'_>, asking: &mut Option<Question>) -> bool {
    let Some(paste) = view.upload.paste() else {
        return running_or_failed(ui, view);
    };

    let ctx = ui.ctx().clone();
    let text = |id: &str| view.text(&ctx, id);
    ui.horizontal(|ui| {
        // As tall as the buttons before the text comes, so it centres on
        // their middle and not on egui's shorter row.
        ui.set_min_height(push_height(ui));
        ui.label(rich(
            ui,
            &text("share-uploaded-short"),
            Weight::SemiBold,
            BODY,
            ui.tokens().text,
        ));

        if pill_button(ui, &text("share-copy-link"), ButtonKind::Primary).clicked() {
            copy(&ctx, view.commands, &paste.link, "share-link-copied");
        }

        if pill_button(ui, &text("share-hide"), ButtonKind::Soft).clicked() {
            view.commands.send(view.hide.clone());
        }

        menu_button(ui, &text("share-link-actions"), |ui| {
            let open = ui.enabled_ui(ours(paste), |ui| menu_item(ui, &text("share-open-link")));

            if open.inner.clicked() {
                links::open(&ctx, &paste.link);
            }

            if menu_item(ui, &text("share-copy-delete-link")).clicked() {
                copy(
                    &ctx,
                    view.commands,
                    &paste.delete_link,
                    "share-delete-link-copied",
                );
            }

            if menu_item(ui, &text("share-delete-now")).clicked() {
                *asking = Some(Question::Delete);
            }
        });
    });

    true
}

/// The running upload or why it failed. Returns whether it drew anything.
fn running_or_failed(ui: &mut Ui, view: &ShareView<'_>) -> bool {
    let ctx = ui.ctx().clone();
    match view.upload {
        Upload::Idle | Upload::Done(_) => return false,
        Upload::Running => {
            loading_label(ui, &view.text(&ctx, "share-uploading"));
        }
        Upload::Failed(problem) => {
            let text = failure(&ctx, view, problem);
            let dismiss = i18n::text(&ctx, "share-upload-dismiss");
            if banner(ui, Tone::Error, &text, &[&dismiss]).value == Some(0) {
                view.commands.send(view.hide.clone());
            }
        }
    }

    true
}

/// Why the upload failed, with the system's words when there are any.
fn failure(ctx: &egui::Context, view: &ShareView<'_>, problem: &UploadProblem) -> String {
    let words = view.text(ctx, failure_id(problem.kind));
    let detail = problem.detail.as_str();
    if detail.is_empty() {
        return words;
    }

    i18n::text_with(
        ctx,
        "share-upload-detail",
        &[
            ("problem", Arg::from(&words)),
            ("detail", Arg::from(detail)),
        ],
    )
    .into_owned()
}

/// Only the paste server's own links open, whatever came back.
fn ours(paste: &Paste) -> bool {
    paste.link.starts_with(&format!("{SERVER}/?"))
}

/// The open question, if any: the upload of `shareable`, or the delete.
pub fn ask(
    ctx: &egui::Context,
    view: &ShareView<'_>,
    shareable: &Redacted,
    asking: &mut Option<Question>,
) -> bool {
    let Some(question) = *asking else {
        return false;
    };

    let words = |id: &str| view.text(ctx, id);

    // The text to check needs the room, the delete only a sentence.
    let (title, cancel, confirm, kind, width) = match question {
        Question::Upload => (
            "share-upload-question",
            "share-upload-cancel",
            "share-upload-confirm",
            ButtonKind::Primary,
            DialogWidth::Wide,
        ),
        Question::Delete => (
            "share-delete-question",
            "share-delete-cancel",
            "share-delete-confirm",
            ButtonKind::Danger,
            DialogWidth::Medium,
        ),
    };

    let title = words(title);
    let close = i18n::text(ctx, "dialog-close");
    let id = Id::unique("toolbox_gui::share::question").with(view.subject.as_str());
    let outcome = ModalWrapper::new(id, &title)
        .title_bar(&close)
        .width(width)
        .show(ctx, |ui| {
            match question {
                Question::Upload => preview::show(ui, view, shareable),
                Question::Delete => {
                    ui.add(Label::new(words("share-delete-detail")).wrap());
                }
            }

            confirm_buttons(ui, &words(cancel), &words(confirm), kind)
        });

    let confirmed = outcome.inner == Some(Confirmation::Confirmed);
    if let Some(paste) = view
        .upload
        .paste()
        .filter(|_| confirmed && question == Question::Delete)
    {
        links::open(ctx, &paste.delete_link);

        // The links lead nowhere once the paste is gone.
        view.commands.send(view.hide.clone());
    }

    if outcome.closed || outcome.inner.is_some() {
        *asking = None;
        preview::forget(ctx, view.subject);
    }

    confirmed && question == Question::Upload
}

pub fn copy(ctx: &egui::Context, commands: &CommandSender, text: &str, id: &str) {
    ctx.copy_text(text.to_owned());
    commands.send(AppCommand::Toast {
        kind: ToastKind::Info,
        text: i18n::text(ctx, id).into_owned(),
    });
}

const fn failure_id(kind: UploadFailure) -> &'static str {
    match kind {
        UploadFailure::Network => "share-upload-network",
        UploadFailure::Refused => "share-upload-refused",
        UploadFailure::BadAnswer => "share-upload-bad-answer",
        UploadFailure::Seal => "share-upload-seal",
        UploadFailure::Stopped => "share-upload-stopped",
    }
}
