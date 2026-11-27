use egui::{Key, Label, Ui};
use toolbox_core::features::users::rows::UserRow;
use toolbox_core::features::users::view::LockAction;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, RowState, Tone, icon_button_width, menu_item, named_pill_button, pill_button,
    pill_width, push_height, rich, row_height, row_menu, split_row_in, status_dot,
    text_block_height_with,
};
use toolbox_plan::Toggle;

use super::dialogs::{Dialog, Edit, Membership, PasswordForm, RemoveUser};
use super::texts;
use crate::command::{UserEdit, UsersCommand};
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::more::text_item;
use crate::users::Users;

const CONTROLS_WIDTH: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.row_actions_narrow;

pub fn height(ui: &Ui) -> f32 {
    row_height(ui, text_block_height_with(ui, 2, push_height(ui)))
}

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (row, state): (&UserRow, &RowState),
    dialog: &mut Option<Dialog>,
) {
    let focused = state.response.has_focus();
    let removing = row.queued.removal.is_some();
    if focused && row.editable && !removing && ui.input(|input| input.key_pressed(Key::Enter)) {
        *dialog = Some(Dialog::Edit(Edit::new(row)));
    }

    split_row_in(
        ui,
        state.row,
        controls_width(ui, removing),
        |ui| words(ui, context, row),
        |ui| controls(ui, context, (row, state), dialog),
    );
}

/// The width the controls take: the button of the row's state (Edit, or
/// Unqueue Removal for a row queued for removal), the gap and the ⋯ button.
/// A reservation for the widest text would push the text under the controls
/// on a narrow window.
pub(super) fn controls_width(ui: &Ui, removing: bool) -> f32 {
    let id = if removing { "users-keep" } else { "users-edit" };
    let button = pill_width(ui, &i18n::text(ui.ctx(), id));
    button + ui.tokens().sizes.control_gap + icon_button_width()
}

fn words(ui: &mut Ui, context: &PageContext<'_>, row: &UserRow) {
    let ctx = ui.ctx().clone();
    let tokens = ui.tokens();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
        let title = rich(
            ui,
            row.user.name.as_str(),
            Weight::SemiBold,
            BODY,
            tokens.text,
        );

        ui.add(Label::new(title));
        let full_name = row.shown_full_name();
        if !full_name.is_empty() {
            ui.add(
                Label::new(rich(ui, full_name, Weight::Regular, BODY, tokens.secondary)).truncate(),
            );
        }

        // Accent marks a queued change on every page, which this is not.
        if row.current {
            status_dot(ui, &text("users-tag-you"), Tone::Neutral);
        }

        if context.service::<Users>().locks().of(&row.user.name) == Some(Toggle::On) {
            status_dot(ui, &text("users-tag-locked"), Tone::Warning);
        }

        if !row.user.is_regular() {
            status_dot(ui, &text("users-tag-system"), Tone::Neutral);
        }

        let queued = row.queued.count();
        if queued > 0 {
            let count = i18n::text_with(&ctx, "users-tag-queued", &[("count", Arg::from(queued))]);

            status_dot(ui, &count, Tone::Accent);
        }
    });

    let not_set = text("users-not-set");
    let shell = Some(row.shown_shell()).filter(|shell| !shell.is_empty());
    let uid = row.user.uid;
    let words = [
        ("uid", Arg::from(uid)),
        ("home", Arg::from(row.shown_home())),
        ("shell", Arg::from(shell.unwrap_or(&not_set))),
    ];

    let detail = i18n::text_with(&ctx, "users-detail", &words);
    super::detail_line(ui, &detail);
    let groups = texts::list(&ctx, row.groups.iter().map(toolbox_plan::Name::as_str));
    let groups = i18n::text_with(&ctx, "users-groups-line", &[("groups", Arg::from(&groups))]);
    super::detail_line(ui, &groups);
}

fn controls(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (row, state): (&UserRow, &RowState),
    dialog: &mut Option<Dialog>,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    if !row.editable {
        // The controls lie in a row that never wraps, so a longer
        // translation ran past the row's edge. Two lines fit its height.
        let note = rich(
            ui,
            &text("users-root"),
            Weight::Regular,
            SMALL,
            ui.tokens().secondary,
        );

        ui.vertical(|ui| {
            ui.set_max_width(CONTROLS_WIDTH);
            ui.add(Label::new(note).wrap());
        });
        return;
    }

    let more = i18n::text_with(
        &ctx,
        "users-more",
        &[("user", Arg::from(row.user.name.as_str()))],
    );

    let name = &row.user.name;

    if row.queued.removal.is_some() {
        if pill_button(ui, &text("users-keep"), ButtonKind::Soft).clicked() {
            context.commands.send(UsersCommand::Keep(name.clone()));
        }
    } else {
        // Every row shows the same word, so each button names its account
        // for a screen reader.
        let user = [("user", Arg::from(name.as_str()))];
        let edit_name = i18n::text_with(&ctx, "users-edit-named", &user);
        let edit = named_pill_button(ui, &text("users-edit"), ButtonKind::Soft, &edit_name);
        if edit.clicked() {
            *dialog = Some(Dialog::Edit(Edit::new(row)));
        }
    }

    row_menu(ui, state, &more, |ui| menu(ui, context, row, dialog));
}

/// An account queued for removal gets no new password.
fn menu(ui: &mut Ui, context: &PageContext<'_>, row: &UserRow, dialog: &mut Option<Dialog>) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let name = &row.user.name;
    let removing = row.queued.removal.is_some();
    if !removing && text_item(ui, &text("users-password"), !context.busy) {
        *dialog = Some(Dialog::Password(PasswordForm::new(name.clone())));
        ui.close();
    }

    if !row.current {
        let known = context.service::<Users>().locks().of(name);
        let action = LockAction::of(row.queued.lock, known);
        let id = match action {
            Some(LockAction::Lock) => "users-lock",
            Some(LockAction::Unlock) => "users-unlock",
            Some(LockAction::UnqueueLock) => "users-unqueue-lock",
            Some(LockAction::UnqueueUnlock) => "users-unqueue-unlock",
            None => "users-lock-unknown",
        };

        let item = ui
            .enabled_ui(action.is_some(), |ui| menu_item(ui, &text(id)))
            .inner;

        if let (true, Some(action)) = (item.clicked(), action) {
            context.commands.send(UserEdit::SetLocked {
                user: name.clone(),
                locked: action.locked(),
            });
            ui.close();
        }
    }

    if menu_item(ui, &text("users-groups")).clicked() {
        *dialog = Some(Dialog::Groups(Membership::new(name.clone())));
        ui.close();
    }

    if !row.current && row.user.is_regular() && menu_item(ui, &text("users-remove")).clicked() {
        *dialog = Some(Dialog::Remove(RemoveUser {
            user: name.clone(),
            home: row.user.home_text().to_owned(),
            groups: row.groups.iter().map(ToString::to_string).collect(),
            delete_home: false,
        }));
        ui.close();
    }
}
