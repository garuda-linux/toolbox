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

use super::Users;
use crate::command::{AppCommand, UserEdit, UsersCommand};
use crate::page_service::Handled;
use crate::refused::{queue, queue_or_refuse};

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
                queue(changes, |changes| keep_user(user, changes))
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
        let subject = match edit {
            UserEdit::SetLocked { user, locked: _ }
            | UserEdit::SetGroups { user, groups: _ }
            | UserEdit::SetFullName { user, name: _ }
            | UserEdit::SetShell { user, shell: _ }
            | UserEdit::SetHome { user, change: _ }
            | UserEdit::Remove { user, home: _ } => user.to_string(),
            UserEdit::AddGroup(group) | UserEdit::RemoveGroup(group) => group.to_string(),
        };

        let locks = &self.locks;
        queue_or_refuse(&mut self.refusal, subject, changes, |queued| match edit {
            UserEdit::SetLocked { user, locked } => {
                let wish = Toggle::from_on(*locked);
                core_users::set_locked(user, wish, locks.of(user), queued, snapshot)
            }
            UserEdit::SetGroups { user, groups } => set_groups(user, groups, queued, snapshot),
            UserEdit::SetFullName { user, name } => {
                set_full_name(user, name.clone(), queued, snapshot)
            }
            UserEdit::SetShell { user, shell } => set_shell(user, shell.clone(), queued, snapshot),
            UserEdit::SetHome { user, change } => set_home(user, change.clone(), queued, snapshot),
            UserEdit::Remove { user, home } => remove_user(user, *home, queued, snapshot),
            UserEdit::AddGroup(group) => add_group(group, queued, snapshot),
            UserEdit::RemoveGroup(group) => remove_group(group, queued, snapshot),
        })
    }
}
