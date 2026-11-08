//! The links hold the key to the paste, so they are only shown and copied
//! when asked, never logged.

use egui::{Id, Label, ScrollArea, TextEdit, Ui};
use toolbox_core::features::diagnostics::UploadFailure;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, Tone, banner, card, confirm_buttons,
    loading_label, menu_button, menu_item, pill_button, rich, text_box, text_field,
};
use toolbox_privatebin::{Paste, SERVER};
use toolbox_system::share::{Upload, UploadProblem};

use crate::command::{AppCommand, Command, CommandSender};
use crate::{i18n, links};

const PREVIEW_HEIGHT: f32 = 240.0;

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
    const fn as_str(self) -> &'static str {
        match self {
            Self::Report => "report",
            Self::Output => "output",
        }
    }
}

impl ShareView<'_> {
    fn text(&self, ctx: &egui::Context, id: &str) -> String {
        i18n::text_with(ctx, id, &[("subject", self.subject.as_str())]).into_owned()
    }
}

/// The link, the failure or the running upload. Nothing before the first
/// upload. Returns whether it drew anything.
pub fn status(ui: &mut Ui, view: &ShareView<'_>, asking: &mut Option<Question>) -> bool {
    let Some(paste) = view.upload.paste() else {
        return running_or_failed(ui, view);
    };

    padded_card(ui, |ui| links_of(ui, view, paste, asking));
    true
}

/// As [`status`] in one row, for a place with little room: Copy link,
/// the other link actions in a ⋯ menu, and Hide.
pub fn row(ui: &mut Ui, view: &ShareView<'_>, asking: &mut Option<Question>) -> bool {
    let Some(paste) = view.upload.paste() else {
        return running_or_failed(ui, view);
    };

    let ctx = ui.ctx().clone();
    let text = |id: &str| view.text(&ctx, id);
    ui.horizontal(|ui| {
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

        menu_button(ui, &text("share-link-actions"), |ui| {
            let open = ui.add_enabled_ui(ours(paste), |ui| menu_item(ui, &text("share-open-link")));

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

        if pill_button(ui, &text("share-hide"), ButtonKind::Soft).clicked() {
            view.commands.send(view.hide.clone());
        }
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
            if banner(ui, Tone::Error, &text, &[&dismiss]) == Some(0) {
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
        &[("problem", &words), ("detail", detail)],
    )
    .into_owned()
}

fn links_of(ui: &mut Ui, view: &ShareView<'_>, paste: &Paste, asking: &mut Option<Question>) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| view.text(&ctx, id);

    field(
        ui,
        &text("share-uploaded"),
        &paste.link,
        &text("share-link"),
    );
    ui.horizontal_wrapped(|ui| {
        if pill_button(ui, &text("share-copy-link"), ButtonKind::Primary).clicked() {
            copy(&ctx, view.commands, &paste.link, "share-link-copied");
        }

        let open = ui.add_enabled_ui(ours(paste), |ui| {
            pill_button(ui, &text("share-open-link"), ButtonKind::Soft)
        });

        if open.inner.clicked() {
            links::open(&ctx, &paste.link);
        }

        if pill_button(ui, &text("share-hide"), ButtonKind::Soft).clicked() {
            view.commands.send(view.hide.clone());
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    let delete = text("share-delete-link");
    field(ui, &delete, &paste.delete_link, &delete);
    ui.horizontal_wrapped(|ui| {
        if pill_button(ui, &text("share-copy-delete-link"), ButtonKind::Soft).clicked() {
            copy(
                &ctx,
                view.commands,
                &paste.delete_link,
                "share-delete-link-copied",
            );
        }

        if pill_button(ui, &text("share-delete-now"), ButtonKind::Soft).clicked() {
            *asking = Some(Question::Delete);
        }
    });
}

/// Only the paste server's own links open, whatever came back.
fn ours(paste: &Paste) -> bool {
    paste.link.starts_with(&format!("{SERVER}/?"))
}

/// A link to read and select, never to edit, with `label` above it.
fn field(ui: &mut Ui, label: &str, link: &str, name: &str) {
    let tokens = ui.tokens();
    ui.add(Label::new(rich(ui, label, Weight::SemiBold, BODY, tokens.text)).wrap());

    // A card leaves no room between its items, the gaps here are the
    // form's own.
    ui.add_space(tokens.sizes.caption_gap);

    // The link is longer than the card, so it scrolls with a bar at rest
    // instead of ending at the field's edge.
    ui.scope(|ui| {
        toolbox_gui_kit::theme::show_scroll_bars(ui.style_mut());
        ScrollArea::horizontal()
            .id_salt(("share-link", name))
            .show(ui, |ui| {
                let mut shown = link;
                let response = text_field(
                    ui,
                    TextEdit::singleline(&mut shown)
                        .clip_text(false)
                        .desired_width(ui.available_width())
                        .font(egui::TextStyle::Monospace)
                        .align(egui::Align2::LEFT_CENTER),
                );

                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::accesskit::Role::TextInput, true, name)
                });
            });
    });

    ui.add_space(tokens.sizes.control_gap);
}

/// The open question, if any: the upload with `text`, or the delete.
/// Returns whether the user agreed to upload `text`.
pub fn ask(
    ctx: &egui::Context,
    view: &ShareView<'_>,
    text: &str,
    asking: &mut Option<Question>,
) -> bool {
    let Some(question) = *asking else {
        return false;
    };

    let words = |id: &str| view.text(ctx, id);
    let (title, cancel, confirm, kind) = match question {
        Question::Upload => (
            "share-upload-question",
            "share-upload-cancel",
            "share-upload-confirm",
            ButtonKind::Primary,
        ),
        Question::Delete => (
            "share-delete-question",
            "share-delete-cancel",
            "share-delete-confirm",
            ButtonKind::Danger,
        ),
    };

    let title = words(title);
    let close = i18n::text(ctx, "dialog-close");
    let id = Id::unique("toolbox_gui::share::question").with(view.subject.as_str());
    let outcome = ModalWrapper::new(id, &title)
        .title_bar(&close)
        .width(DialogWidth::Medium)
        .show(ctx, |ui| {
            match question {
                Question::Upload => preview(ui, view, text),
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
    }

    if outcome.closed || outcome.inner.is_some() {
        *asking = None;
    }

    confirmed && question == Question::Upload
}

fn preview(ui: &mut Ui, view: &ShareView<'_>, text: &str) {
    let ctx = ui.ctx().clone();
    ui.add(Label::new(view.text(&ctx, "share-upload-where")).wrap());
    ui.add(Label::new(view.text(&ctx, "share-upload-check")).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);
    let name = view.text(&ctx, "share-upload-text");
    let mut shown = text;
    text_box(
        ui,
        TextEdit::multiline(&mut shown).font(egui::TextStyle::Monospace),
        &name,
        PREVIEW_HEIGHT,
    );
}

pub fn copy(ctx: &egui::Context, commands: &CommandSender, text: &str, id: &str) {
    ctx.copy_text(text.to_owned());
    commands.send(AppCommand::Toast {
        kind: ToastKind::Info,
        text: i18n::text(ctx, id).into_owned(),
    });
}

pub fn padded_card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    let padding = ui.tokens().sizes.card_padding;
    card(ui, |ui| {
        egui::Frame::new()
            .inner_margin(padding)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                body(ui)
            })
            .inner
    })
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
