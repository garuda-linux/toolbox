//! The Users and groups page's clicks that edit the queue. A refused one
//! stays on the page until dismissed or the next click is taken.

#[cfg(test)]
mod tests;

use toolbox_core::features::users::home::set_home;
use toolbox_core::features::users::{
    self as core_users, add_group, keep_user, remove_group, remove_user, set_full_name, set_groups,
    set_shell,
};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_services::queue::edit::QueueEdit;

use super::Users;
use crate::command::{AppCommand, UserEdit, UsersCommand};
use crate::page_service::Handled;
use crate::refused::queue_or_refuse;

impl Users {
    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Users(command) = command else {
            return Handled::default();
        };

        match command {
            UsersCommand::Keep(user) => {
                self.refusal = None;
                Handled::edit(QueueEdit::of_click(changes, |changes| keep_user(user, changes)).0)
            }
            UsersCommand::DismissRefusal => {
                self.refusal = None;
                Handled::taken()
            }
            UsersCommand::Edit(edit) => self.edit(edit, changes, snapshot),
            UsersCommand::Refresh
            | UsersCommand::AddUser {
                user: _,
                password: _,
            }
            | UsersCommand::SetPassword {
                user: _,
                password: _,
            }
            | UsersCommand::DismissRun
            | UsersCommand::ShowConsole => Handled::default(),
        }
    }

    fn edit(&mut self, edit: &UserEdit, changes: &ChangeSet, snapshot: &Snapshot) -> Handled {
        let slot = &mut self.refusal;
        match edit {
            UserEdit::SetLocked { user, locked } => {
                let known = self.locks.of(user);
                let wish = Toggle::from_on(*locked);
                queue_or_refuse(slot, user.to_string(), changes, |queued| {
                    core_users::set_locked(user, wish, known, queued, snapshot)
                })
            }
            UserEdit::SetGroups { user, groups } => {
                queue_or_refuse(slot, user.to_string(), changes, |queued| {
                    set_groups(user, groups, queued, snapshot)
                })
            }
            UserEdit::SetFullName { user, name } => {
                queue_or_refuse(slot, user.to_string(), changes, |queued| {
                    set_full_name(user, name.clone(), queued, snapshot)
                })
            }
            UserEdit::SetShell { user, shell } => {
                queue_or_refuse(slot, user.to_string(), changes, |queued| {
                    set_shell(user, shell.clone(), queued, snapshot)
                })
            }
            UserEdit::SetHome { user, change } => {
                queue_or_refuse(slot, user.to_string(), changes, |queued| {
                    set_home(user, change.clone(), queued, snapshot)
                })
            }
            UserEdit::Remove { user, home } => {
                queue_or_refuse(slot, user.to_string(), changes, |queued| {
                    remove_user(user, *home, queued, snapshot)
                })
            }
            UserEdit::AddGroup(group) => {
                queue_or_refuse(slot, group.to_string(), changes, |queued| {
                    add_group(group, queued, snapshot)
                })
            }
            UserEdit::RemoveGroup(group) => {
                queue_or_refuse(slot, group.to_string(), changes, |queued| {
                    remove_group(group, queued, snapshot)
                })
            }
        }
    }
}
