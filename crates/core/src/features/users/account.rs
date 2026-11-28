//! The Edit dialog's full name, login shell and home folder as one change.
//! Every guard runs before anything is queued, so a refused home folder
//! does not leave the typed name queued alone.

use toolbox_plan::{FullName, HomeChange, HomeDirectory, InvalidFullName, ShellPath, UserName};

use super::home::check_home;
use super::input::{self, InputProblem};
use super::{Refusal, accounts, editable, shell_is_listed};
use crate::domains::{FullNames, HomeFolders, LoginShells};
use crate::{ChangeSet, Snapshot};

/// What changes on one account, `None` where it stays.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountEdit {
    pub full_name: Option<FullName>,
    pub shell: Option<ShellPath>,
    pub home: Option<HomeChange>,
}

/// The text of the Edit dialog's fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fields {
    pub full_name: String,
    pub shell: String,
    pub home: String,
    pub move_files: bool,
}

/// The typed values, each `None` while unchanged.
#[derive(Debug)]
pub struct Changes {
    pub full_name: Option<Result<FullName, InvalidFullName>>,
    pub shell: Option<ShellPath>,
    pub home: Option<Result<HomeDirectory, InputProblem>>,
}

impl Changes {
    /// The fields `now` changed from `start`. `current_home` is the folder
    /// the account has.
    pub fn between(start: &Fields, now: &Fields, current_home: &str) -> Self {
        // The move flag alone changes only a queued move: the files of the
        // folder the account has stay where they are.
        let elsewhere = now.home != current_home;
        let moved = elsewhere && now.move_files != start.move_files;

        Self {
            full_name: (now.full_name != start.full_name)
                .then(|| FullName::new(now.full_name.clone())),
            shell: (now.shell != start.shell)
                .then(|| ShellPath::new(&now.shell).ok())
                .flatten(),
            home: (now.home != start.home || moved).then(|| input::home(&now.home)),
        }
    }

    pub const fn any(&self) -> bool {
        self.full_name.is_some() || self.shell.is_some() || self.home.is_some()
    }

    /// The edit the changes make, `None` while a typed value is invalid.
    pub fn edit(&self, move_files: bool) -> Option<AccountEdit> {
        let full_name = match &self.full_name {
            Some(Ok(name)) => Some(name.clone()),
            Some(Err(_)) => return None,
            None => None,
        };

        let home = match &self.home {
            Some(Ok(home)) => Some(HomeChange {
                home: home.clone(),
                move_files,
            }),
            Some(Err(_)) => return None,
            None => None,
        };

        Some(AccountEdit {
            full_name,
            shell: self.shell.clone(),
            home,
        })
    }
}

/// Whether [`set_account`] would take `edit`.
///
/// # Errors
///
/// The first [`Refusal`] of a part: root, an unknown user, a shell
/// `/etc/shells` does not list, or a home folder that may not move there.
pub fn check_account(
    user: &UserName,
    edit: &AccountEdit,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    editable(user, accounts(snapshot)?)?;

    if let Some(shell) = &edit.shell
        && !shell_is_listed(shell, snapshot)
    {
        return Err(Refusal::ShellNotListed);
    }

    if let Some(change) = &edit.home {
        check_home(user, &change.home, snapshot)?;
    }

    Ok(())
}

/// Queues every part of `edit`, or nothing when one part is refused. A
/// value the account has now takes a queued change of it back.
///
/// # Errors
///
/// As [`check_account`].
pub fn set_account(
    user: &UserName,
    edit: AccountEdit,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    check_account(user, &edit, snapshot)?;

    let AccountEdit {
        full_name,
        shell,
        home,
    } = edit;

    if let Some(full_name) = full_name {
        changes.want::<FullNames>(user.clone(), full_name, snapshot);
    }

    if let Some(shell) = shell {
        changes.want::<LoginShells>(user.clone(), shell, snapshot);
    }

    if let Some(home) = home {
        changes.want::<HomeFolders>(user.clone(), home, snapshot);
    }

    Ok(())
}
