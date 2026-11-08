//! The groups of one user as check boxes. The primary group stays ticked,
//! and there is no way to tick every group at once: that would also make
//! the user an administrator.

use std::collections::BTreeSet;

use egui::{Context, Ui};
use toolbox_core::features::users::rows::GroupChoice;
use toolbox_core::features::users::view;
use toolbox_gui_kit::widgets::{ButtonKind, check_row};
use toolbox_plan::{GroupName, UserName};

use super::{Next, dialog, note};
use crate::command::UserEdit;
use crate::i18n;
use crate::pages::PageContext;

#[derive(Debug)]
pub struct Membership {
    user: UserName,
    chosen: Option<BTreeSet<GroupName>>,
}

impl Membership {
    pub const fn new(user: UserName) -> Self {
        Self { user, chosen: None }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-groups-title", &[("user", user)]);
        let confirm = i18n::text(ctx, "users-queue-changes");
        let choices = view::choices(&self.user, context.snapshot, context.changes);
        let chosen = self.chosen.get_or_insert_with(|| {
            choices
                .iter()
                .filter(|choice| choice.member)
                .map(|choice| choice.group.clone())
                .collect()
        });

        let (next, confirmed) = dialog(
            ctx,
            "membership",
            (&title, &confirm, ButtonKind::Primary),
            true,
            |ui| {
                note(ui, &i18n::text(ctx, "users-groups-note"));
                ui.add_space(ui.spacing().item_spacing.y);
                for choice in &choices {
                    tick(ui, choice, chosen);
                }
            },
        );

        if confirmed {
            context.commands.send(UserEdit::SetGroups {
                user: self.user.clone(),
                groups: chosen.clone(),
            });
        }

        next
    }
}

fn tick(ui: &mut Ui, choice: &GroupChoice, chosen: &mut BTreeSet<GroupName>) {
    let ctx = ui.ctx().clone();
    let members = choice.members.to_string();
    let detail = choice.gid.map_or_else(
        || i18n::text(&ctx, "users-choice-new").into_owned(),
        |gid| {
            let id = if choice.primary {
                "users-choice-primary"
            } else {
                "users-choice-detail"
            };

            let gid = gid.to_string();
            i18n::text_with(&ctx, id, &[("gid", &gid), ("members", &members)]).into_owned()
        },
    );

    let on = choice.primary || chosen.contains(&choice.group);
    let label = format!("{} ({detail})", choice.group);
    if check_row(ui, on, &label, "", !choice.primary).clicked() {
        if on {
            chosen.remove(&choice.group);
        } else {
            chosen.insert(choice.group.clone());
        }
    }
}
