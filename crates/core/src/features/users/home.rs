//! Where accounts live: the home folder and user id a new account asks
//! for, and a new home folder for an existing one.

use toolbox_plan::{HomeChange, HomeDirectory, NewHome, NewUser, NewUserParts, UserId, UserName};

use super::guards::{in_user_range, is_current};
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
/// [`Refusal`] for root, an account outside the user ids of people, an
/// unknown user, the app's own user (usermod refuses an account that
/// runs programs) or the home folder of another account.
pub fn set_home(
    user: &UserName,
    change: HomeChange,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    check_home(user, &change.home, snapshot)?;
    changes.want::<HomeFolders>(user.clone(), change, snapshot);
    Ok(())
}

/// Whether the home folder of `user` may move at all, before a path is
/// typed, so the Edit dialog can say so on the field.
///
/// # Errors
///
/// As [`set_home`], without [`Refusal::HomeTaken`].
pub fn home_movable(user: &UserName, snapshot: &Snapshot) -> Result<(), Refusal> {
    in_user_range(editable(user, accounts(snapshot)?)?)?;

    if is_current(user, snapshot) {
        return Err(Refusal::CurrentUser);
    }

    Ok(())
}

/// Whether `home` may become the home folder of `user`.
///
/// # Errors
///
/// As [`set_home`].
pub fn check_home(
    user: &UserName,
    home: &HomeDirectory,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    home_movable(user, snapshot)?;

    if is_taken(home, Some(user), accounts(snapshot)?) {
        return Err(Refusal::HomeTaken);
    }

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
        .any(|user| user.home_text() == home.as_str() && Some(&user.name) != owner)
}
