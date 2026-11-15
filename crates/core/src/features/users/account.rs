//! The Edit dialog's full name, login shell and home folder as one change.
//! Every guard runs before anything is queued, so a refused home folder
//! does not leave the typed name queued alone.

use toolbox_plan::{FullName, HomeChange, ShellPath, UserName};

use super::home::check_home;
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
