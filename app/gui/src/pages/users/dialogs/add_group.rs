use egui::Context;
use toolbox_core::features::users::{input, view};
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::GroupName;

use super::{Next, dialog, field, note};
use crate::command::UserEdit;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::users::texts;

#[derive(Debug, Default)]
pub struct AddGroup {
    name: String,
}

impl AddGroup {
    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let title = i18n::text(ctx, "groups-add-title");
        let confirm = i18n::text(ctx, "groups-add-confirm");
        let checked = check(&self.name, context);
        let problem = match &checked {
            Err(code) if !self.name.is_empty() => Some(texts::problem(ctx, code, &self.name)),
            Ok(_) | Err(_) => None,
        };

        let name = &mut self.name;
        let (next, confirmed) = dialog(
            ctx,
            "add-group",
            (&title, &confirm, ButtonKind::Primary),
            checked.is_ok(),
            |ui| {
                let label = i18n::text(ctx, "groups-field-name");
                let hint = i18n::text(ctx, "users-field-name-hint");
                field(ui, (&label, &hint), name, problem.as_deref());
                note(ui, &i18n::text(ctx, "groups-add-note"));
            },
        );

        if confirmed && let Ok(group) = checked {
            context.commands.send(UserEdit::AddGroup(group));
        }

        next
    }
}

fn check(text: &str, context: &PageContext<'_>) -> Result<GroupName, &'static str> {
    let group: GroupName = input::name(text)?;
    let taken = view::groups(context.snapshot, context.changes, &view::Shown::default())
        .iter()
        .any(|row| row.name == group && row.gid.is_some());

    if taken {
        return Err("group-exists");
    }

    Ok(group)
}
