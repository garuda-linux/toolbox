//! Creates a user and sets its password at once. Each field says what is
//! wrong with it while it is typed. Core builds the account from the
//! fields, so the dialog and the queue apply the same rules.
//!
//! The full name comes first and proposes the login until that is typed
//! by hand. A failed run opens the dialog again as it was sent, without
//! the passwords, which went with the run.

mod more;

use std::collections::BTreeSet;
use std::fmt;

use egui::{Context, Ui};
use toolbox_core::features::users::home::check_uid;
use toolbox_core::features::users::input::{
    self, AddForm, InputProblem, NewAccount, add_blocker, login_from_full_name,
};
use toolbox_core::features::users::now::check_new_name;
use toolbox_core::features::users::run::{Retry, RunKind};
use toolbox_core::features::users::view;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{ButtonKind, disclosure};
use toolbox_plan::{FullName, GroupName, InvalidFullName, NewUser, UserName};

use super::password::PasswordPair;
use super::shell::{ShellDefault, shell_choice};
use super::{Next, check_with_note, dialog_why, field, note};
use crate::command::UsersCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::users::texts;

#[derive(Default)]
pub struct AddUser {
    full_name: String,
    name: String,
    /// Whether the login was typed by hand, so the full name no longer
    /// proposes it.
    login: Login,
    /// Empty for the system's default shell.
    shell: String,
    home: String,
    uid: String,
    no_home: bool,
    system: bool,
    pub(super) password: PasswordPair,
    admin: bool,
    groups: BTreeSet<GroupName>,
    search: String,
    /// The account the dialog sent when it closed, `None` before.
    sent: Option<UserName>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Login {
    /// Follows the full name.
    #[default]
    Proposed,
    Typed,
}

/// What each field says is wrong with it, `None` where nothing is.
#[derive(Default)]
struct Problems {
    name: Option<String>,
    full_name: Option<String>,
    home: Option<String>,
    uid: Option<String>,
}

impl fmt::Debug for AddUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddUser")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl AddUser {
    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let title = i18n::text(ctx, "users-add-title");
        let confirm = i18n::text(ctx, "users-add-confirm");
        let (account, problems) = self.check(ctx, context);
        let agreed = self.password.agreed();
        let ready = account.is_ok() && agreed && !context.busy;
        let why = self.why(ctx, context, &problems, agreed);
        let (next, confirmed) = dialog_why(
            ctx,
            "add-user",
            (&title, &confirm, ButtonKind::Primary),
            (why.as_deref(), ready),
            |ui| {
                // It is about the whole dialog. Under the last option it
                // read as that option's description.
                note(ui, &i18n::text(ctx, "users-add-note"));
                ui.add_space(ui.spacing().item_spacing.y);
                self.required(ui, ctx, context, &problems);

                // Kept by egui, so the dialog opens again as it was left.
                let id = egui::Id::unique("toolbox_gui::users::add_more");
                let mut open = ui.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
                let more = i18n::text(ctx, "users-add-more");
                if disclosure(ui, &more, open).clicked() {
                    open = !open;
                    ui.data_mut(|data| data.insert_temp(id, open));
                }

                if open {
                    ui.add_space(ui.tokens().sizes.card_gap);
                    shell_choice(ui, context, &mut self.shell, ShellDefault::Offered);
                    self.place(ui, ctx, &problems);
                    self.other_groups(ui, ctx, context);
                }
            },
        );

        if confirmed && let (Ok(user), Some(password)) = (account, self.password.password()) {
            self.sent = Some(user.parts().name.clone());
            context.commands.send(UsersCommand::AddUser {
                user: Box::new(user),
                password,
            });
        }

        next
    }

    /// The fields as sent, without the passwords, to open again when the
    /// run fails. `None` when the dialog closed without sending.
    pub fn retry(&self) -> Option<Retry<Box<Self>>> {
        let user = self.sent.clone()?;
        let form = Self {
            full_name: self.full_name.clone(),
            name: self.name.clone(),
            login: self.login,
            shell: self.shell.clone(),
            home: self.home.clone(),
            uid: self.uid.clone(),
            no_home: self.no_home,
            system: self.system,
            admin: self.admin,
            groups: self.groups.clone(),
            ..Self::default()
        };

        Some(Retry::new(Box::new(form), RunKind::NewAccount, user))
    }

    /// The account the fields make, and each field's problem.
    fn check(
        &self,
        ctx: &Context,
        context: &PageContext<'_>,
    ) -> (Result<NewUser, InputProblem>, Problems) {
        let snapshot = context.snapshot;
        let named = |problem: InputProblem| texts::problem(ctx, problem, &self.name);
        let name = input::name(&self.name).and_then(|user: UserName| {
            check_new_name(&user, snapshot).map_err(InputProblem::Refused)
        });

        let uid = input::user_id(&self.uid)
            .and_then(|uid| check_uid(uid, self.system, snapshot).map_err(InputProblem::Refused));

        let problems = Problems {
            name: name.err().filter(|_| !self.name.is_empty()).map(named),
            full_name: FullName::new(self.full_name.clone())
                .map_err(InvalidFullName::as_str)
                .err()
                .map(|code| texts::full_name_problem(ctx, code)),
            home: (!self.no_home && !self.home.is_empty())
                .then(|| input::home(&self.home).err())
                .flatten()
                .map(named),
            uid: uid.err().map(named),
        };

        let mut groups = self.groups.clone();
        if self.admin
            && let Some(admin) = view::admin_group(snapshot, context.changes)
        {
            groups.insert(admin.clone());
        }

        let form = NewAccount {
            name: &self.name,
            full_name: &self.full_name,
            shell: &self.shell,
            groups: Some(groups),
            home: &self.home,
            no_home: self.no_home,
            uid: &self.uid,
            system: self.system,
        };

        (input::new_account(&form, snapshot), problems)
    }

    /// Why "Add user now" waits, `None` once it can be clicked.
    fn why(
        &self,
        ctx: &Context,
        context: &PageContext<'_>,
        problems: &Problems,
        password_ok: bool,
    ) -> Option<String> {
        let (first, second) = self.password.texts();
        let form = AddForm {
            name: &self.name,
            name_ok: problems.name.is_none(),
            first,
            second,
            password_ok,
            rest_ok: problems.full_name.is_none()
                && problems.home.is_none()
                && problems.uid.is_none(),
        };

        let code = match add_blocker(&form) {
            Some(blocker) => blocker.as_str(),
            None if context.busy => "busy",
            None => return None,
        };

        Some(i18n::text(ctx, &format!("users-add-why-{code}")).into_owned())
    }

    fn required(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        context: &PageContext<'_>,
        problems: &Problems,
    ) {
        let text = |id: &str| i18n::text(ctx, id).into_owned();
        let full_labels = (
            text("users-field-full-name"),
            text("users-field-full-name-hint"),
        );

        let full_name = field(
            ui,
            (&full_labels.0, &full_labels.1),
            &mut self.full_name,
            problems.full_name.as_deref(),
        );

        if full_name.changed() && self.login == Login::Proposed {
            self.name = login_from_full_name(&self.full_name);
        }

        let login_labels = (text("users-field-name"), text("users-field-name-hint"));
        let login = field(
            ui,
            (&login_labels.0, &login_labels.1),
            &mut self.name,
            problems.name.as_deref(),
        );

        // Emptied by hand, the login follows the full name again.
        if login.changed() {
            self.login = if self.name.is_empty() {
                Login::Proposed
            } else {
                Login::Typed
            };
        }

        self.password.show(ui, ctx);

        if let Some(group) = view::admin_group(context.snapshot, context.changes) {
            let label = text("users-admin");
            let detail = i18n::text_with(
                ctx,
                "users-admin-detail",
                &[("group", Arg::from(group.as_str()))],
            );

            if check_with_note(ui, self.admin, &label, &detail) {
                self.admin = !self.admin;
            }

            ui.add_space(ui.tokens().sizes.card_gap);
        }
    }
}
