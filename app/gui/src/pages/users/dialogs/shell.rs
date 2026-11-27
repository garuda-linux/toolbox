//! The login shell dropdown of Add user and Edit.

use egui::Ui;
use toolbox_core::features::users::view;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::Select;
use toolbox_plan::ShellPath;

use super::{note, title};
use crate::i18n;
use crate::pages::PageContext;

/// Whether the dropdown starts with the system's default shell, which a
/// new account may take and an existing one already has a path for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShellDefault {
    Offered,
    Left,
}

/// The shells `/etc/shells` lists, after the default when it is offered.
/// Empty `chosen` is the default.
pub(super) fn shell_choice(
    ui: &mut Ui,
    context: &PageContext<'_>,
    chosen: &mut String,
    default: ShellDefault,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let shells = view::shells(context.snapshot);
    let default_name = text("users-shell-default");
    let shown = ShellPath::names(shells);
    let skip = usize::from(default == ShellDefault::Offered);
    let names: Vec<&str> = std::iter::once(default_name.as_str())
        .chain(shown.iter().map(String::as_str))
        .skip(1 - skip)
        .collect();

    let picked = shells.iter().position(|shell| shell.as_str() == chosen);
    let current = if chosen.is_empty() && default == ShellDefault::Offered {
        Some(0)
    } else {
        picked.map(|index| index + skip)
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
        .value
    {
        *chosen = index
            .checked_sub(skip)
            .and_then(|index| shells.get(index))
            .map(|shell| shell.as_str().to_owned())
            .unwrap_or_default();
    }

    note(ui, &text("users-field-shell-hint"));
    ui.add_space(ui.tokens().sizes.card_gap);
}
