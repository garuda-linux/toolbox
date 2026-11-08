//! Sets a user's password at once. The two fields live only here, and go
//! with the dialog when it closes.

use std::fmt;

use egui::Context;
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::{InvalidPassword, Password, UserName};

use super::{Next, dialog, note, password_field};
use crate::command::UsersCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::users::texts;

pub struct PasswordForm {
    user: UserName,
    first: String,
    second: String,
}

impl fmt::Debug for PasswordForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasswordForm")
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}

impl PasswordForm {
    pub const fn new(user: UserName) -> Self {
        Self {
            user,
            first: String::new(),
            second: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-password-title", &[("user", user)]);
        let confirm = i18n::text(ctx, "users-password-confirm");
        let (password, problems) = checked(ctx, &self.first, &self.second);
        let ready = password.is_some() && !context.busy;
        let (first, second) = (&mut self.first, &mut self.second);
        let (next, confirmed) = dialog(
            ctx,
            "password",
            (&title, &confirm, ButtonKind::Primary),
            ready,
            |ui| {
                note(ui, &i18n::text(ctx, "users-password-note"));
                ui.add_space(ui.spacing().item_spacing.y);
                let first_label = i18n::text(ctx, "users-field-password");
                password_field(ui, &first_label, first, problems.0.as_deref());
                let second_label = i18n::text(ctx, "users-field-repeat");
                password_field(ui, &second_label, second, problems.1.as_deref());
            },
        );

        if confirmed && let Some(password) = password {
            context.commands.send(UsersCommand::SetPassword {
                user: self.user.clone(),
                password,
            });
        }

        next
    }
}

/// A password both fields agree on, and the problem of each field. A field
/// that is still empty names none.
pub fn checked(
    ctx: &Context,
    first: &str,
    second: &str,
) -> (Option<Password>, (Option<String>, Option<String>)) {
    let password = Password::new(first.to_owned()).map_err(InvalidPassword::as_str);
    let first_problem = match &password {
        Err(code) if !first.is_empty() => Some(texts::password_problem(ctx, code)),
        Ok(_) | Err(_) => None,
    };

    let differ = first != second;
    let second_problem =
        (differ && !second.is_empty()).then(|| texts::password_problem(ctx, "mismatch"));

    let password = password.ok().filter(|_| !differ);
    (password, (first_problem, second_problem))
}
