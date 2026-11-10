use egui::{Key, Label, Ui};
use toolbox_core::features::users::rows::UserRow;
use toolbox_core::features::users::view::LockAction;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, RowState, Tone, menu_item, named_pill_button, pill_button, rich, row_height,
    row_menu, split_row, status_dot, text_block_height,
};

use super::dialogs::{Dialog, Edit, Membership, PasswordForm, RemoveUser};
use super::texts;
use crate::command::{UserEdit, UsersCommand};
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::more::text_item;
use crate::users::Users;

const CONTROLS_WIDTH: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.row_actions_narrow;

pub fn height(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, 2))
}

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (row, state): (&UserRow<'_>, &RowState),
    dialog: &mut Option<Dialog>,
) {
    let focused = state.response.has_focus();
    if focused && row.editable && ui.input(|input| input.key_pressed(Key::Enter)) {
        *dialog = Some(Dialog::Edit(Edit::new(row)));
    }

    split_row(
        ui,
        CONTROLS_WIDTH,
        |ui| words(ui, context, row),
        |ui| controls(ui, context, (row, state), dialog),
    );
}

fn words(ui: &mut Ui, context: &PageContext<'_>, row: &UserRow<'_>) {
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

        if row.current {
            status_dot(ui, &text("users-tag-you"), Tone::Accent);
        }

        if context.service::<Users>().locks().of(&row.user.name) == Some(true) {
            status_dot(ui, &text("users-tag-locked"), Tone::Warning);
        }

        if !row.user.is_regular() {
            status_dot(ui, &text("users-tag-system"), Tone::Neutral);
        }

        let queued = row.queued.count();
        if queued > 0 {
            let count =
                i18n::text_with(&ctx, "users-tag-queued", &[("count", &queued.to_string())]);

            status_dot(ui, &count, Tone::Accent);
        }
    });

    let not_set = text("users-not-set");
    let shell = Some(row.shown_shell()).filter(|shell| !shell.is_empty());
    let uid = row.user.uid.to_string();
    let words = [
        ("uid", uid.as_str()),
        ("home", row.shown_home()),
        ("shell", shell.unwrap_or(&not_set)),
    ];

    let detail = i18n::text_with(&ctx, "users-detail", &words);
    ui.add(Label::new(rich(ui, &detail, Weight::Regular, SMALL, tokens.secondary)).truncate());
    let groups = texts::list(&ctx, row.groups.iter().map(toolbox_plan::Name::as_str));
    let groups = i18n::text_with(&ctx, "users-groups-line", &[("groups", &groups)]);
    ui.add(Label::new(rich(ui, &groups, Weight::Regular, SMALL, tokens.secondary)).truncate());
}

fn controls(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (row, state): (&UserRow<'_>, &RowState),
    dialog: &mut Option<Dialog>,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    if !row.editable {
        ui.label(rich(
            ui,
            &text("users-root"),
            Weight::Regular,
            SMALL,
            ui.tokens().secondary,
        ));
        return;
    }

    let more = i18n::text_with(&ctx, "users-more", &[("user", row.user.name.as_str())]);
    let name = &row.user.name;
    if row.queued.removal.is_some() {
        if pill_button(ui, &text("users-keep"), ButtonKind::Soft).clicked() {
            context.commands.send(UsersCommand::Keep(name.clone()));
        }
    } else {
        // Every row shows the same word, so each button names its account
        // for a screen reader.
        let user = [("user", name.as_str())];
        let edit_name = i18n::text_with(&ctx, "users-edit-named", &user);
        let edit = named_pill_button(ui, &text("users-edit"), ButtonKind::Soft, &edit_name);
        if edit.clicked() {
            *dialog = Some(Dialog::Edit(Edit::new(row)));
        }
    }

    ui.add_space(ui.tokens().sizes.control_gap);
    row_menu(ui, state, &more, |ui| menu(ui, context, row, dialog));
}

/// An account queued for removal gets no new password.
fn menu(ui: &mut Ui, context: &PageContext<'_>, row: &UserRow<'_>, dialog: &mut Option<Dialog>) {
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
            .add_enabled_ui(action.is_some(), |ui| menu_item(ui, &text(id)))
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
            home: row.user.home.clone(),
            groups: row.groups.iter().map(ToString::to_string).collect(),
            delete_home: false,
        }));
        ui.close();
    }
}
