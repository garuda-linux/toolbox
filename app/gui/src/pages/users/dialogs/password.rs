//! Sets a user's password at once. The two fields live only here, and go
//! with the dialog when it closes.

use std::fmt;

use egui::{Context, Id, TextBuffer, Ui};
use toolbox_gui_kit::widgets::{ButtonKind, SecretText, forget_text};
use toolbox_plan::{InvalidPassword, MAX_PASSWORD_BYTES, Password, UserName};

use super::{Next, dialog, note, password_field};
use crate::command::UsersCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::users::texts;

pub struct PasswordForm {
    user: UserName,
    pub(super) pair: PasswordPair,
}

impl fmt::Debug for PasswordForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasswordForm")
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}

impl PasswordForm {
    pub fn new(user: UserName) -> Self {
        Self {
            user,
            pair: PasswordPair::default(),
        }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-password-title", &[("user", user)]);
        let confirm = i18n::text(ctx, "users-password-confirm");
        let ready = self.pair.agreed() && !context.busy;
        let pair = &mut self.pair;
        let (next, confirmed) = dialog(
            ctx,
            "password",
            (&title, &confirm, ButtonKind::Primary),
            ready,
            |ui| {
                note(ui, &i18n::text(ctx, "users-password-note"));
                ui.add_space(ui.spacing().item_spacing.y);
                pair.show(ui, ctx);
            },
        );

        if confirmed && let Some(password) = self.pair.password() {
            context.commands.send(UsersCommand::SetPassword {
                user: self.user.clone(),
                password,
            });
        }

        next
    }
}

/// A password and its repetition. A `Password` is built only when a field
/// changes and when the dialog confirms, as each one is another copy of
/// the text.
pub struct PasswordPair {
    first: SecretText,
    second: SecretText,
    check: Check,
    fields: Vec<Id>,
}

/// The error code of each field, `None` where nothing is wrong or nothing
/// was typed yet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Check {
    agreed: bool,
    first: Option<&'static str>,
    second: Option<&'static str>,
}

impl PasswordPair {
    pub fn show(&mut self, ui: &mut Ui, ctx: &Context) {
        let problem = |code: Option<&str>| code.map(|code| texts::password_problem(ctx, code));
        let first_label = i18n::text(ctx, "users-field-password");
        let second_label = i18n::text(ctx, "users-field-repeat");
        let first_problem = problem(self.check.first);
        let second_problem = problem(self.check.second);
        let first = password_field(ui, &first_label, &mut self.first, first_problem.as_deref());
        let second = password_field(
            ui,
            &second_label,
            &mut self.second,
            second_problem.as_deref(),
        );

        if first.changed() || second.changed() {
            self.check = check(self.first.as_str(), self.second.as_str());
        }

        for id in [first.id, second.id] {
            if !self.fields.contains(&id) {
                self.fields.push(id);
            }
        }
    }

    pub const fn agreed(&self) -> bool {
        self.check.agreed
    }

    pub fn texts(&self) -> (&str, &str) {
        (self.first.as_str(), self.second.as_str())
    }

    pub fn password(&self) -> Option<Password> {
        check(self.first.as_str(), self.second.as_str())
            .agreed
            .then(|| Password::new(self.first.as_str().to_owned()).ok())
            .flatten()
    }

    /// Drops the text and what egui kept of both fields.
    pub fn forget(&mut self, ctx: &Context) {
        for id in self.fields.drain(..) {
            forget_text(ctx, id);
        }

        TextBuffer::clear(&mut self.first);
        TextBuffer::clear(&mut self.second);
        self.check = Check::default();
    }
}

impl Default for PasswordPair {
    fn default() -> Self {
        Self {
            first: SecretText::with_room(MAX_PASSWORD_BYTES),
            second: SecretText::with_room(MAX_PASSWORD_BYTES),
            check: Check::default(),
            fields: Vec::new(),
        }
    }
}

fn check(first: &str, second: &str) -> Check {
    let valid = Password::new(first.to_owned()).map_err(InvalidPassword::as_str);
    let differ = first != second;
    Check {
        agreed: valid.is_ok() && !differ,
        first: valid.err().filter(|_| !first.is_empty()),
        second: (differ && !second.is_empty()).then_some("mismatch"),
    }
}
