//! Queues a new full name, login shell or home folder for one user. The
//! fields start with what the row shows: the queued value, else the
//! current one. Only what changed is queued.

use egui::{Context, Ui};
use toolbox_core::features::users::rows::UserRow;
use toolbox_core::features::users::{input, view};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{ButtonKind, Select, check_row};
use toolbox_plan::{FullName, HomeChange, HomeDirectory, InvalidFullName, ShellPath, UserName};

use super::{Next, dialog, field, note, title};
use crate::command::UserEdit;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::users::texts;

#[derive(Debug)]
pub struct Edit {
    user: UserName,
    /// The account the app runs for: its home folder cannot change.
    current: bool,
    start: Fields,
    fields: Fields,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fields {
    full_name: String,
    shell: String,
    home: String,
    move_files: bool,
}

/// The typed values, each `None` while unchanged.
struct Changes {
    full_name: Option<Result<FullName, &'static str>>,
    shell: Option<ShellPath>,
    home: Option<Result<HomeDirectory, &'static str>>,
}

impl Edit {
    pub fn new(row: &UserRow<'_>) -> Self {
        let start = Fields {
            full_name: row.shown_full_name().to_owned(),
            shell: row.shown_shell().to_owned(),
            home: row.shown_home().to_owned(),
            move_files: row.queued.home.as_ref().is_some_and(|home| home.move_files),
        };

        Self {
            user: row.user.name.clone(),
            current: row.current,
            fields: start.clone(),
            start,
        }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-edit-title", &[("user", user)]);
        let confirm = i18n::text(ctx, "users-queue-changes");
        let changes = self.changes();
        let ready = !matches!(changes.full_name, Some(Err(_)))
            && !matches!(changes.home, Some(Err(_)))
            && self.fields != self.start;

        let (next, confirmed) = dialog(
            ctx,
            "edit",
            (&title, &confirm, ButtonKind::Primary),
            ready,
            |ui| {
                self.body(ui, ctx, context, &changes);
            },
        );

        if confirmed {
            self.send(context, changes);
        }

        next
    }

    fn changes(&self) -> Changes {
        let (now, start) = (&self.fields, &self.start);
        let moved = now.move_files != start.move_files;
        Changes {
            full_name: (now.full_name != start.full_name)
                .then(|| FullName::new(now.full_name.clone()).map_err(InvalidFullName::as_str)),
            shell: (now.shell != start.shell)
                .then(|| ShellPath::new(&now.shell).ok())
                .flatten(),
            home: (now.home != start.home || moved).then(|| input::home(&now.home)),
        }
    }

    fn body(&mut self, ui: &mut Ui, ctx: &Context, context: &PageContext<'_>, changes: &Changes) {
        let text = |id: &str| i18n::text(ctx, id).into_owned();
        let problem = match &changes.full_name {
            Some(Err(code)) => Some(texts::full_name_problem(ctx, code)),
            Some(Ok(_)) | None => None,
        };

        let label = text("users-field-full-name");
        field(
            ui,
            (&label, ""),
            &mut self.fields.full_name,
            problem.as_deref(),
        );
        shell(ui, ctx, context, &mut self.fields.shell);
        if self.current {
            note(ui, &text("users-home-current-user"));
        } else {
            self.home(ui, ctx, changes);
        }

        ui.add_space(ui.spacing().item_spacing.y);
        note(ui, &text("users-edit-note"));
    }

    fn home(&mut self, ui: &mut Ui, ctx: &Context, changes: &Changes) {
        let problem = match &changes.home {
            Some(Err(code)) => Some(texts::problem(ctx, code, "")),
            Some(Ok(_)) | None => None,
        };

        let label = i18n::text(ctx, "users-field-home");
        let hint = i18n::text(ctx, "users-field-home-edit-hint");
        field(
            ui,
            (&label, &hint),
            &mut self.fields.home,
            problem.as_deref(),
        );
        let move_label = i18n::text(ctx, "users-move-files");
        let moving = self.fields.move_files;
        if check_row(ui, moving, &move_label, "", true).clicked() {
            self.fields.move_files = !moving;
        }

        let id = if self.fields.move_files {
            "users-move-yes"
        } else {
            "users-move-no"
        };

        let words = [
            ("old", self.start.home.as_str()),
            ("new", self.fields.home.as_str()),
            ("user", self.user.as_str()),
        ];

        note(ui, &i18n::text_with(ctx, id, &words));
    }

    fn send(&self, context: &PageContext<'_>, changes: Changes) {
        let user = &self.user;
        if let Some(Ok(name)) = changes.full_name {
            context.commands.send(UserEdit::SetFullName {
                user: user.clone(),
                name,
            });
        }

        if let Some(shell) = changes.shell {
            context.commands.send(UserEdit::SetShell {
                user: user.clone(),
                shell,
            });
        }

        if let Some(Ok(home)) = changes.home {
            context.commands.send(UserEdit::SetHome {
                user: user.clone(),
                change: HomeChange {
                    home,
                    move_files: self.fields.move_files,
                },
            });
        }
    }
}

/// The login shell, one of those `/etc/shells` lists.
fn shell(ui: &mut Ui, ctx: &Context, context: &PageContext<'_>, chosen: &mut String) {
    let shells = view::shells(context.snapshot);
    let shown = ShellPath::names(shells);
    let names: Vec<&str> = shown.iter().map(String::as_str).collect();
    let current = shells.iter().position(|shell| shell.as_str() == chosen);
    let label = i18n::text(ctx, "users-field-shell");
    title(ui, &label);
    if let Some(picked) = Select::new(&label, &names)
        .selected(current)
        .detail(
            current
                .and_then(|index| shells.get(index))
                .map(ShellPath::as_str),
        )
        .width(ui.available_width())
        .show(ui)
        .and_then(|index| shells.get(index))
    {
        picked.as_str().clone_into(chosen);
    }

    note(ui, &i18n::text(ctx, "users-field-shell-hint"));
    ui.add_space(ui.tokens().sizes.card_gap);
}
