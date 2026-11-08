//! Where accounts live: the home folder and user id a new account asks
//! for, and a new home folder for an existing one.

use toolbox_plan::{HomeChange, HomeDirectory, NewHome, NewUser, NewUserParts, UserId, UserName};

use super::guards::is_current;
use super::{Refusal, accounts, editable};
use crate::domains::{Accounts, HomeFolders};
use crate::{ChangeSet, Snapshot};

/// What the add-user dialog collected beside name, shell and groups.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    pub home: NewHome,
    pub uid: Option<UserId>,
    pub system: bool,
}

/// `user` with `placement`, checked against the system.
///
/// # Errors
///
/// [`Refusal::UidOutOfRange`] for a user id outside the range of the
/// kind of account, [`Refusal::UidInUse`] for one an account has, and
/// [`Refusal::HomeTaken`] for the home folder of another account.
pub fn place(user: NewUser, placement: Placement, snapshot: &Snapshot) -> Result<NewUser, Refusal> {
    let accounts = accounts(snapshot)?;
    check_uid(placement.uid, placement.system, snapshot)?;

    if let NewHome::At(home) = &placement.home
        && is_taken(home, None, accounts)
    {
        return Err(Refusal::HomeTaken);
    }

    NewUser::new(NewUserParts {
        home: placement.home,
        uid: placement.uid,
        system: placement.system,
        ..user.into_parts()
    })
    .map_err(|_out_of_range| Refusal::UidOutOfRange)
}

/// Whether a new account of the given kind may ask for `uid`.
///
/// # Errors
///
/// [`Refusal::UidOutOfRange`], [`Refusal::UidInUse`] or
/// [`Refusal::Unavailable`].
pub fn check_uid(uid: Option<UserId>, system: bool, snapshot: &Snapshot) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;
    let Some(uid) = uid else {
        return Ok(());
    };

    if uid.is_system() != system {
        return Err(Refusal::UidOutOfRange);
    }

    if accounts.users.iter().any(|user| user.uid == uid.get()) {
        return Err(Refusal::UidInUse);
    }

    Ok(())
}

/// Queues a new home folder for `user`. The path the account has now
/// takes a queued change back.
///
/// # Errors
///
/// [`Refusal`] for root, an unknown user, the app's own user (usermod
/// refuses an account that runs programs) or the home folder of another
/// account.
pub fn set_home(
    user: &UserName,
    change: HomeChange,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    editable(user, accounts)?;

    if is_current(user, snapshot) {
        return Err(Refusal::CurrentUser);
    }

    if is_taken(&change.home, Some(user), accounts) {
        return Err(Refusal::HomeTaken);
    }

    changes.want::<HomeFolders>(user.clone(), change, snapshot);
    Ok(())
}

pub(super) fn is_taken(
    home: &HomeDirectory,
    owner: Option<&UserName>,
    accounts: &Accounts,
) -> bool {
    accounts
        .users
        .iter()
        .any(|user| user.home == home.as_str() && Some(&user.name) != owner)
}
