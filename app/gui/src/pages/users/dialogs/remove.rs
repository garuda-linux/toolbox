//! Asks before a user or a group is queued to go, and says what goes.

use egui::{Context, Label, Ui};
use toolbox_gui_kit::widgets::{ButtonKind, check_row};
use toolbox_plan::{GroupName, HomeFolder, UserName};

use super::{Next, dialog, note};
use crate::command::UserEdit;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::users::texts;

/// The user to remove, with what the dialog lists. The home folder is
/// kept unless ticked, each time the dialog opens.
#[derive(Debug)]
pub struct RemoveUser {
    pub user: UserName,
    pub home: String,
    pub groups: Vec<String>,
    pub delete_home: bool,
}

#[derive(Debug)]
pub struct RemoveGroup(pub GroupName);

impl RemoveUser {
    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-remove-title", &[("user", user)]);
        let confirm = i18n::text(ctx, "users-remove-confirm");
        let (next, confirmed) = dialog(
            ctx,
            "remove-user",
            (&title, &confirm, ButtonKind::Danger),
            true,
            |ui| {
                self.body(ui, ctx);
            },
        );

        if confirmed {
            let home = if self.delete_home {
                HomeFolder::Delete
            } else {
                HomeFolder::Keep
            };

            context.commands.send(UserEdit::Remove {
                user: self.user.clone(),
                home,
            });
        }

        next
    }

    fn body(&mut self, ui: &mut Ui, ctx: &Context) {
        let user = self.user.as_str();
        let home = self.home.as_str();
        let with = |id: &str, key: &'static str, value: &str| {
            format!("• {}", i18n::text_with(ctx, id, &[(key, value)]))
        };

        ui.add(Label::new(i18n::text(ctx, "users-remove-goes")).wrap());
        ui.add(Label::new(with("users-remove-account", "user", user)).wrap());
        ui.add(Label::new(with("users-remove-group", "user", user)).wrap());
        if !self.groups.is_empty() {
            let groups = texts::list(ctx, &self.groups);
            ui.add(Label::new(with("users-remove-memberships", "groups", &groups)).wrap());
        }

        if self.delete_home {
            ui.add(Label::new(with("users-remove-home-goes", "home", home)).wrap());
        }

        ui.add_space(ui.spacing().item_spacing.y);
        let label = i18n::text(ctx, "users-remove-delete-home");
        if check_row(ui, self.delete_home, &label, "", true).clicked() {
            self.delete_home = !self.delete_home;
        }

        if !self.delete_home {
            note(
                ui,
                &i18n::text_with(ctx, "users-remove-home-stays", &[("home", home)]),
            );
        }
    }
}

impl RemoveGroup {
    pub fn show(&self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let group = self.0.as_str();
        let title = i18n::text_with(ctx, "groups-remove-title", &[("group", group)]);
        let confirm = i18n::text(ctx, "groups-remove-confirm");
        let (next, confirmed) = dialog(
            ctx,
            "remove-group",
            (&title, &confirm, ButtonKind::Danger),
            true,
            |ui| {
                ui.add(Label::new(i18n::text(ctx, "groups-remove-detail")).wrap());
            },
        );

        if confirmed {
            context.commands.send(UserEdit::RemoveGroup(self.0.clone()));
        }

        next
    }
}
