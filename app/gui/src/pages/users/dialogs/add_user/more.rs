//! The fields behind More options: home folder, user id, the kind of
//! account and the other groups.

use egui::{Context, Ui};
use toolbox_core::features::users::view;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::check_row;

use super::super::{check_with_note, field, group_search, title};
use super::{AddUser, Problems};
use crate::i18n;
use crate::pages::PageContext;

impl AddUser {
    pub(super) fn place(&mut self, ui: &mut Ui, ctx: &Context, problems: &Problems) {
        let text = |id: &str| i18n::text(ctx, id).into_owned();
        if !self.no_home {
            let labels = (text("users-field-home"), text("users-field-home-hint"));
            field(
                ui,
                (&labels.0, &labels.1),
                &mut self.home,
                problems.home.as_deref(),
            );
        }

        let labels = (text("users-field-uid"), text("users-field-uid-hint"));
        field(
            ui,
            (&labels.0, &labels.1),
            &mut self.uid,
            problems.uid.as_deref(),
        );
        for (on, id) in [
            (&mut self.no_home, "users-no-home"),
            (&mut self.system, "users-system"),
        ] {
            let detail = text(&format!("{id}-detail"));
            if check_with_note(ui, *on, &text(id), &detail) {
                *on = !*on;
            }
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    /// The groups besides the administrators.
    pub(super) fn other_groups(&mut self, ui: &mut Ui, ctx: &Context, context: &PageContext<'_>) {
        let admin = view::admin_group(context.snapshot, context.changes);
        title(ui, &i18n::text(ctx, "users-other-groups"));
        group_search(ui, &mut self.search);
        let names = view::group_names(context.snapshot, context.changes, &self.search);

        // Rows of the dialog's own scrolling body, never a list scrolling
        // inside it: the search above narrows a system's dozens of groups.
        for group in names.iter().filter(|group| Some(*group) != admin) {
            let on = self.groups.contains(group);
            let box_row = check_row(ui, on, group.as_str(), "", true);
            if box_row.gained_focus() {
                box_row.scroll_to_me(None);
            }

            if box_row.clicked() {
                if on {
                    self.groups.remove(group);
                } else {
                    self.groups.insert(group.clone());
                }
            }
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}
