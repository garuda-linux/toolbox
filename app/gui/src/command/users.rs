//! Each queue edit names the account or group it is for, so a refusal can name it too.

use std::collections::BTreeSet;

use toolbox_plan::{
    FullName, GroupName, HomeChange, HomeFolder, NewUser, Password, ShellPath, UserName,
};

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UsersCommand {
    Refresh,
    Edit(UserEdit),
    Keep(UserName),
    /// Creates the account and sets its password at once. The password
    /// goes from here into the plan and nowhere else.
    AddUser {
        user: Box<NewUser>,
        password: Password,
    },
    SetPassword {
        user: UserName,
        password: Password,
    },
    DismissRun,
    ShowConsole,
    DismissRefusal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserEdit {
    SetLocked {
        user: UserName,
        locked: bool,
    },
    SetGroups {
        user: UserName,
        groups: BTreeSet<GroupName>,
    },
    SetFullName {
        user: UserName,
        name: FullName,
    },
    SetShell {
        user: UserName,
        shell: ShellPath,
    },
    SetHome {
        user: UserName,
        change: HomeChange,
    },
    /// Queues the removal of `user`, keeping or deleting its home folder.
    Remove {
        user: UserName,
        home: HomeFolder,
    },
    AddGroup(GroupName),
    RemoveGroup(GroupName),
}

impl From<UsersCommand> for Command {
    fn from(command: UsersCommand) -> Self {
        Self::App(AppCommand::Users(command))
    }
}

impl From<UserEdit> for Command {
    fn from(edit: UserEdit) -> Self {
        UsersCommand::Edit(edit).into()
    }
}
