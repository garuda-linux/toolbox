//! The fields behind More options: login shell, home folder, user id, the
//! kind of account and the other groups.

use egui::{Context, TextEdit, Ui};
use toolbox_core::features::users::view;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{Select, check_row, text_field};
use toolbox_plan::ShellPath;

use super::super::{field, note, title};
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
            if check_row(ui, *on, &text(id), &detail, true).clicked() {
                *on = !*on;
            }

            note(ui, &detail);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    /// The groups besides the administrators.
    pub(super) fn other_groups(&mut self, ui: &mut Ui, ctx: &Context, context: &PageContext<'_>) {
        let admin = view::admin_group(context.snapshot, context.changes);
        title(ui, &i18n::text(ctx, "users-other-groups"));
        let search = text_field(
            ui,
            TextEdit::singleline(&mut self.search)
                .hint_text(i18n::text(ctx, "users-groups-search-hint"))
                .desired_width(f32::INFINITY)
                .align(egui::Align2::LEFT_CENTER),
        );

        let name = i18n::text(ctx, "users-groups-search").into_owned();
        ui.ctx()
            .accesskit_node_builder(search.id, |node| node.set_label(name));

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

/// The login shell: the system's default first, then what `/etc/shells`
/// lists. Empty `chosen` is the default.
pub(super) fn shell_choice(
    ui: &mut Ui,
    ctx: &Context,
    context: &PageContext<'_>,
    chosen: &mut String,
) {
    let text = |id: &str| i18n::text(ctx, id).into_owned();
    let shells = view::shells(context.snapshot);
    let default = text("users-shell-default");
    let shown = ShellPath::names(shells);
    let names: Vec<&str> = std::iter::once(default.as_str())
        .chain(shown.iter().map(String::as_str))
        .collect();

    let picked = shells.iter().position(|shell| shell.as_str() == chosen);
    let current = if chosen.is_empty() {
        Some(0)
    } else {
        picked.map(|index| index + 1)
    };

    let label = text("users-field-shell");
    title(ui, &label);
    if let Some(index) = Select::new(&label, &names)
        .selected(current)
        .detail(
            picked
                .and_then(|index| shells.get(index))
                .map(ShellPath::as_str),
        )
        .width(ui.available_width())
        .show(ui)
    {
        *chosen = index
            .checked_sub(1)
            .and_then(|index| shells.get(index))
            .map(|shell| shell.as_str().to_owned())
            .unwrap_or_default();
    }

    note(ui, &text("users-field-shell-hint"));
    ui.add_space(ui.tokens().sizes.card_gap);
}
