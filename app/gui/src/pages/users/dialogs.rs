//! The page's dialogs. One is open at a time, and each keeps what is typed
//! into it until it closes. A password lives only in its dialog and in the
//! command that sends it.

mod add_group;
mod add_user;
mod edit;
mod membership;
mod password;
mod remove;

use egui::{Context, Id, Label, Response, TextEdit, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, SecretText, confirm_buttons_when,
    confirm_buttons_why, rich, text_field,
};

pub use self::add_group::AddGroup;
pub use self::add_user::AddUser;
pub use self::edit::Edit;
pub use self::membership::Membership;
pub use self::password::PasswordForm;
pub use self::remove::{RemoveGroup, RemoveUser};
use crate::i18n;
use crate::pages::PageContext;

const BUTTONS_HEIGHT: f32 = 80.0;

#[derive(Debug)]
pub enum Dialog {
    AddUser(Box<AddUser>),
    Edit(Edit),
    Password(PasswordForm),
    Groups(Membership),
    Remove(RemoveUser),
    AddGroup(AddGroup),
    RemoveGroup(RemoveGroup),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Stay,
    Close,
}

impl Dialog {
    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        match self {
            Self::AddUser(dialog) => dialog.show(ctx, context),
            Self::Edit(dialog) => dialog.show(ctx, context),
            Self::Password(dialog) => dialog.show(ctx, context),
            Self::Groups(dialog) => dialog.show(ctx, context),
            Self::Remove(dialog) => dialog.show(ctx, context),
            Self::AddGroup(dialog) => dialog.show(ctx, context),
            Self::RemoveGroup(dialog) => dialog.show(ctx, context),
        }
    }

    /// Runs before the dialog is dropped, so no password outlives it in
    /// egui's memory.
    pub fn forget(&mut self, ctx: &Context) {
        match self {
            Self::AddUser(dialog) => dialog.password.forget(ctx),
            Self::Password(dialog) => dialog.pair.forget(ctx),
            Self::Edit(_)
            | Self::Groups(_)
            | Self::Remove(_)
            | Self::AddGroup(_)
            | Self::RemoveGroup(_) => {}
        }
    }
}

pub fn dialog(
    ctx: &Context,
    salt: &str,
    question: (&str, &str, ButtonKind),
    ready: bool,
    body: impl FnOnce(&mut Ui),
) -> (Next, bool) {
    dialog_why(ctx, salt, question, (None, ready), body)
}

pub fn dialog_why(
    ctx: &Context,
    salt: &str,
    (title, confirm, kind): (&str, &str, ButtonKind),
    (why, ready): (Option<&str>, bool),
    body: impl FnOnce(&mut Ui),
) -> (Next, bool) {
    let close = i18n::text(ctx, "dialog-close");
    let cancel = i18n::text(ctx, "users-cancel");
    let outcome = ModalWrapper::new(Id::unique(("toolbox_gui::users", salt)), title)
        .title_bar(&close)
        .width(DialogWidth::Medium)
        .show(ctx, |ui| {
            // A floating bar would cover the right end of the fields.
            ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - BUTTONS_HEIGHT)
                .auto_shrink([false, true])
                .show(ui, body);

            match why.filter(|_| !ready) {
                Some(why) => confirm_buttons_why(ui, &cancel, confirm, kind, Some(why)),
                None => confirm_buttons_when(ui, &cancel, confirm, kind, ready),
            }
        });

    let confirmed = ready && outcome.inner == Some(Confirmation::Confirmed);
    let closed = outcome.closed || outcome.inner.is_some();
    (if closed { Next::Close } else { Next::Stay }, confirmed)
}

pub fn field(
    ui: &mut Ui,
    (label, hint): (&str, &str),
    text: &mut String,
    problem: Option<&str>,
) -> Response {
    field_with(ui, (label, hint), problem, |ui| {
        text_field(
            ui,
            TextEdit::singleline(text)
                .desired_width(f32::INFINITY)
                .align(egui::Align2::LEFT_CENTER),
        )
    })
}

pub fn password_field(
    ui: &mut Ui,
    label: &str,
    text: &mut SecretText,
    problem: Option<&str>,
) -> Response {
    field_with(ui, (label, ""), problem, |ui| {
        text_field(
            ui,
            TextEdit::singleline(text)
                .password(true)
                .desired_width(f32::INFINITY)
                .align(egui::Align2::LEFT_CENTER),
        )
    })
}

fn field_with(
    ui: &mut Ui,
    (label, hint): (&str, &str),
    problem: Option<&str>,
    edit: impl FnOnce(&mut Ui) -> Response,
) -> Response {
    let tokens = ui.tokens();
    title(ui, label);
    let response = edit(ui);
    let name = label.to_owned();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(name));

    let (note, color) = problem.map_or((hint, tokens.secondary), |problem| (problem, tokens.error));

    if !note.is_empty() {
        ui.add(Label::new(rich(ui, note, Weight::Regular, SMALL, color)).wrap());
    }

    ui.add_space(tokens.sizes.card_gap);
    response
}

pub fn note(ui: &mut Ui, text: &str) {
    let secondary = ui.tokens().secondary;
    ui.add(Label::new(rich(ui, text, Weight::Regular, SMALL, secondary)).wrap());
}

pub fn title(ui: &mut Ui, text: &str) {
    let color = ui.tokens().text;
    ui.label(rich(ui, text, Weight::SemiBold, BODY, color));
}
