//! What the app does with a click on the Users and groups page. Most
//! clicks edit the queue, a refused one stays on the page until dismissed
//! or the next click is taken. A new account and a new password run at
//! once, with the Console closed: the page shows how they went.

use toolbox_core::features::users::home::set_home;
use toolbox_core::features::users::now::{plan_add_user, plan_set_password};
use toolbox_core::features::users::run::RunKind;
use toolbox_core::features::users::{
    self as core_users, Refusal, add_group, keep_user, remove_group, remove_user, set_full_name,
    set_groups, set_shell,
};
use toolbox_core::{ChangeSet, Snapshot, Toggle};

use super::App;
use crate::command::{UserEdit, UsersCommand};
use crate::page::PageId;
use crate::pages::UsersRefusal;

impl App {
    pub(super) fn run_users(&mut self, command: UsersCommand) {
        match command {
            UsersCommand::Refresh => {
                self.read_system();
                self.users.refresh();
            }
            UsersCommand::Keep(user) => {
                let snapshot = self.current_snapshot();
                self.queue
                    .edit(&snapshot, |changes, _| keep_user(&user, changes));
                self.refusals.users = None;
            }
            UsersCommand::AddUser { user, password } => {
                let name = user.parts().name.clone();
                if self.run_unseen(plan_add_user(*user, password)) {
                    self.users.started(RunKind::NewAccount, name);
                }
            }
            UsersCommand::SetPassword { user, password } => {
                let snapshot = self.current_snapshot();
                let result = plan_set_password(&user, password, &snapshot).map(|plan| {
                    if self.run_unseen(plan) {
                        self.users.started(RunKind::Password, user.clone());
                    }
                });

                self.refuse_users(result, user.to_string());
            }
            UsersCommand::DismissRun => self.users.dismiss_run(),
            UsersCommand::ShowConsole => self.state.console_open = true,
            UsersCommand::DismissRefusal => self.refusals.users = None,
            UsersCommand::Edit(edit) => self.edit_users(edit),
        }
    }

    fn edit_users(&mut self, edit: UserEdit) {
        match edit {
            UserEdit::SetLocked { user, locked } => {
                let known = self.users.locks().of(&user);
                let wish = Toggle::from_on(locked);
                self.queue_users(user.to_string(), |changes, snapshot| {
                    core_users::set_locked(&user, wish, known, changes, snapshot)
                });
            }
            UserEdit::SetGroups { user, groups } => {
                self.queue_users(user.to_string(), |changes, snapshot| {
                    set_groups(&user, &groups, changes, snapshot)
                });
            }
            UserEdit::SetFullName { user, name } => {
                self.queue_users(user.to_string(), |changes, snapshot| {
                    set_full_name(&user, name, changes, snapshot)
                });
            }
            UserEdit::SetShell { user, shell } => {
                self.queue_users(user.to_string(), |changes, snapshot| {
                    set_shell(&user, shell, changes, snapshot)
                });
            }
            UserEdit::SetHome { user, change } => {
                self.queue_users(user.to_string(), |changes, snapshot| {
                    set_home(&user, change, changes, snapshot)
                });
            }
            UserEdit::Remove { user, home } => {
                self.queue_users(user.to_string(), |changes, snapshot| {
                    remove_user(&user, home, changes, snapshot)
                });
            }
            UserEdit::AddGroup(group) => {
                self.queue_users(group.to_string(), |changes, snapshot| {
                    add_group(&group, changes, snapshot)
                });
            }
            UserEdit::RemoveGroup(group) => {
                self.queue_users(group.to_string(), |changes, snapshot| {
                    remove_group(&group, changes, snapshot)
                });
            }
        }
    }

    fn queue_users<F>(&mut self, subject: String, edit: F)
    where
        F: FnOnce(&mut ChangeSet, &Snapshot) -> Result<(), Refusal>,
    {
        let snapshot = self.current_snapshot();
        let result = self.queue.edit(&snapshot, edit);
        self.refuse_users(result, subject);
    }

    /// The page asks the helper once it shows, and the snapshot takes the
    /// flags that came.
    pub(super) fn take_users(&mut self) {
        if self.state.page == PageId::Users {
            self.users.open();
        }

        if self.users.take() {
            self.combine();
            self.queue.meet(&self.seen);
        }
    }

    fn refuse_users(&mut self, result: Result<(), Refusal>, subject: String) {
        self.refusals.users = result.err().map(|refusal| UsersRefusal {
            code: refusal.as_str(),
            subject,
        });
    }
}
