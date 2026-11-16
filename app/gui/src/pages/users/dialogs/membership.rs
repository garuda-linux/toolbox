//! The groups of one user as check boxes. The primary group stays ticked,
//! and there is no way to tick every group at once: that would also make
//! the user an administrator.

use std::collections::BTreeSet;

use egui::{Context, Ui};
use toolbox_core::features::users::rows::GroupChoice;
use toolbox_core::features::users::view;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{ButtonKind, check_row};
use toolbox_plan::{GroupName, UserName};

use super::{Next, dialog, group_search, note};
use crate::command::UserEdit;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;

#[derive(Debug)]
pub struct Membership {
    user: UserName,
    /// The groups when the dialog opened, then as ticked.
    picks: Option<Picks>,
    search: String,
}

#[derive(Debug)]
struct Picks {
    start: BTreeSet<GroupName>,
    chosen: BTreeSet<GroupName>,
}

impl Membership {
    pub const fn new(user: UserName) -> Self {
        Self {
            user,
            picks: None,
            search: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-groups-title", &[("user", Arg::from(user))]);
        let confirm = i18n::text(ctx, "users-queue-changes");
        let choices = view::choices(&self.user, context.snapshot, context.changes);
        let picks = self.picks.get_or_insert_with(|| {
            let start: BTreeSet<GroupName> = choices
                .iter()
                .filter(|choice| choice.member)
                .map(|choice| choice.group.clone())
                .collect();

            Picks {
                chosen: start.clone(),
                start,
            }
        });

        let search = &mut self.search;
        let ready = picks.chosen != picks.start;
        let (next, confirmed) = dialog(
            ctx,
            "membership",
            (&title, &confirm, ButtonKind::Primary),
            ready,
            |ui| {
                note(ui, &i18n::text(ctx, "users-groups-note"));
                ui.add_space(ui.spacing().item_spacing.y);
                group_search(ui, search);
                for choice in view::choices_holding(&choices, search) {
                    tick(ui, choice, &mut picks.chosen);
                }
            },
        );

        if confirmed {
            context.commands.send(UserEdit::SetGroups {
                user: self.user.clone(),
                groups: picks.chosen.clone(),
            });
        }

        next
    }
}

fn tick(ui: &mut Ui, choice: &GroupChoice, chosen: &mut BTreeSet<GroupName>) {
    let ctx = ui.ctx().clone();
    let group = choice.group.as_str();
    let members = choice.members;
    let label = choice.gid.map_or_else(
        || i18n::text_with(&ctx, "users-choice-new", &[("group", Arg::from(group))]).into_owned(),
        |gid| {
            let id = if choice.primary {
                "users-choice-primary"
            } else {
                "users-choice-detail"
            };

            let words = [
                ("group", Arg::from(group)),
                ("gid", Arg::from(gid)),
                ("members", Arg::from(members)),
            ];

            i18n::text_with(&ctx, id, &words).into_owned()
        },
    );

    let on = choice.primary || chosen.contains(&choice.group);
    let row = ui.enabled_ui(!choice.primary, |ui| check_row(ui, on, &label, ""));
    if row.inner.clicked() {
        if on {
            chosen.remove(&choice.group);
        } else {
            chosen.insert(choice.group.clone());
        }
    }
}
