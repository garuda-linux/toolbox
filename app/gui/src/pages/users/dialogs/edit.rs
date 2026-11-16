//! Queues a new full name, login shell or home folder for one user. The
//! fields start with what the row shows: the queued value, else the
//! current one. Only what changed is queued, as one change: a part the
//! page refuses keeps the others from queueing.
//!
//! The guards run while the dialog shows, so a home folder that cannot
//! move says why in its place, and a refused value holds the button.

use egui::{Context, Ui};
use toolbox_core::features::users::Refusal;
use toolbox_core::features::users::account::{AccountEdit, check_account};
use toolbox_core::features::users::home::home_movable;
use toolbox_core::features::users::input::{self, InputProblem};
use toolbox_core::features::users::rows::UserRow;
use toolbox_gui_kit::widgets::{ButtonKind, check_row};
use toolbox_plan::{FullName, HomeChange, HomeDirectory, InvalidFullName, ShellPath, UserName};

use super::shell::{ShellDefault, shell_choice};
use super::{Next, dialog_why, field, note, title};
use crate::command::UserEdit;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::users::texts;

#[derive(Debug)]
pub struct Edit {
    user: UserName,
    /// The folder the account has now, before any queued move.
    current_home: String,
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
    home: Option<Result<HomeDirectory, InputProblem>>,
}

impl Changes {
    const fn any(&self) -> bool {
        self.full_name.is_some() || self.shell.is_some() || self.home.is_some()
    }
}

impl Edit {
    pub fn new(row: &UserRow) -> Self {
        let start = Fields {
            full_name: row.shown_full_name().to_owned(),
            shell: row.shown_shell().to_owned(),
            home: row.shown_home().to_owned(),
            move_files: row.queued.home.as_ref().is_some_and(|home| home.move_files),
        };

        Self {
            user: row.user.name.clone(),
            current_home: row.user.home.clone(),
            fields: start.clone(),
            start,
        }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let user = self.user.as_str();
        let title = i18n::text_with(ctx, "users-edit-title", &[("user", Arg::from(user))]);
        let confirm = i18n::text(ctx, "users-queue-changes");
        let changes = self.changes();
        let edit = self.account(&changes);
        let refusal = edit
            .as_ref()
            .and_then(|typed| check_account(&self.user, typed, context.snapshot).err());

        let why = refusal.map(|code| refused(ctx, code, user));
        let ready = matches!((&edit, refusal), (Some(_), None)) && changes.any();
        let (next, confirmed) = dialog_why(
            ctx,
            "edit",
            (&title, &confirm, ButtonKind::Primary),
            (why.as_deref(), ready),
            |ui| {
                self.body(ui, ctx, context, &changes);
            },
        );

        if confirmed && let Some(edit) = edit {
            context.commands.send(UserEdit::Set {
                user: self.user.clone(),
                edit,
            });
        }

        next
    }

    fn changes(&self) -> Changes {
        let (now, start) = (&self.fields, &self.start);

        // The move flag alone changes only a queued move: the files of the
        // folder the account has stay where they are.
        let elsewhere = now.home != self.current_home;
        let moved = elsewhere && now.move_files != start.move_files;
        Changes {
            full_name: (now.full_name != start.full_name)
                .then(|| FullName::new(now.full_name.clone()).map_err(InvalidFullName::as_str)),
            shell: (now.shell != start.shell)
                .then(|| ShellPath::new(&now.shell).ok())
                .flatten(),
            home: (now.home != start.home || moved).then(|| input::home(&now.home)),
        }
    }

    /// The edit the fields make, `None` while a typed value is invalid.
    fn account(&self, changes: &Changes) -> Option<AccountEdit> {
        let full_name = match &changes.full_name {
            Some(Ok(name)) => Some(name.clone()),
            Some(Err(_)) => return None,
            None => None,
        };

        let home = match &changes.home {
            Some(Ok(home)) => Some(HomeChange {
                home: home.clone(),
                move_files: self.fields.move_files,
            }),
            Some(Err(_)) => return None,
            None => None,
        };

        Some(AccountEdit {
            full_name,
            shell: changes.shell.clone(),
            home,
        })
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
        shell_choice(ui, context, &mut self.fields.shell, ShellDefault::Left);
        match home_movable(&self.user, context.snapshot) {
            Ok(()) => self.home(ui, ctx, changes),
            Err(refusal) => {
                title(ui, &text("users-field-home"));
                note(ui, &home_refused(ctx, refusal, self.user.as_str()));
            }
        }

        ui.add_space(ui.spacing().item_spacing.y);
        note(ui, &text("users-edit-note"));
    }

    fn home(&mut self, ui: &mut Ui, ctx: &Context, changes: &Changes) {
        let problem = match &changes.home {
            Some(Err(problem)) => Some(texts::problem(ctx, *problem, "")),
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

        // Moving means nothing until the folder differs from the one the
        // account has.
        if self.fields.home == self.current_home {
            return;
        }

        let move_label = i18n::text(ctx, "users-move-files");
        let moving = self.fields.move_files;
        if check_row(ui, moving, &move_label, "").clicked() {
            self.fields.move_files = !moving;
        }

        let id = if self.fields.move_files {
            "users-move-yes"
        } else {
            "users-move-no"
        };

        let words = [
            ("old", Arg::from(self.current_home.as_str())),
            ("new", Arg::from(self.fields.home.as_str())),
            ("user", Arg::from(self.user.as_str())),
        ];

        note(ui, &i18n::text_with(ctx, id, &words));
    }
}

fn refused(ctx: &Context, refusal: Refusal, user: &str) -> String {
    texts::problem(ctx, InputProblem::Refused(refusal), user)
}

/// Why the home folder stays, said where its field would be.
fn home_refused(ctx: &Context, refusal: Refusal, user: &str) -> String {
    if refusal == Refusal::CurrentUser {
        i18n::text(ctx, "users-home-current-user").into_owned()
    } else {
        refused(ctx, refusal, user)
    }
}
