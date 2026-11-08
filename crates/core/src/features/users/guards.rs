//! The guards behind the page's refusals, shared by the clicks and by the
//! recheck of a restored queue.

use std::collections::BTreeSet;

use toolbox_plan::procedure::FIRST_USER_ID;
use toolbox_plan::{GroupName, LAST_USER_ID, UserName};

use super::{ADMIN_GROUPS, Refusal, accounts, editable};
use crate::domains::{
    AccountLockFlags, AccountLocks, Accounts, CurrentUser, Groups, Membership, RemovedUsers, User,
};
use crate::{ChangeSet, Snapshot, Toggle};

/// The range the helper accepts for a lock or a home folder move.
pub(super) fn in_user_range(user: &User) -> Result<(), Refusal> {
    if (FIRST_USER_ID..=LAST_USER_ID).contains(&user.uid) {
        Ok(())
    } else {
        Err(Refusal::OutsideUserRange)
    }
}

/// Whether `user` may be removed. A removal of `user` that is queued
/// already must be taken out first, or the user never counts as an admin.
///
/// # Errors
///
/// [`Refusal`] for root, the app's own user, a system account, an unknown
/// user or the last member of [`ADMIN_GROUPS`].
pub(super) fn removable(
    user: &UserName,
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;
    let account = editable(user, accounts)?;

    if is_current(user, snapshot) {
        return Err(Refusal::CurrentUser);
    }

    if !account.is_regular() {
        return Err(Refusal::SystemAccount);
    }

    if takes_last_admin(user, None, changes, snapshot) {
        return Err(Refusal::LastAdmin);
    }

    Ok(())
}

/// Whether the password of `user` may get the lock state `wish`.
///
/// # Errors
///
/// [`Refusal`] for root, an unknown user, locking an account outside the
/// user ids of people or the app's own user, locking a member of
/// [`ADMIN_GROUPS`] while the admins cannot be told,
/// [`Refusal::LockStateUnknown`] while their lock flags cannot, or
/// locking the last of them.
pub(super) fn lockable(
    user: &UserName,
    wish: Toggle,
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    let account = editable(user, accounts)?;

    if !wish.is_on() {
        return Ok(());
    }

    in_user_range(account)?;

    if is_current(user, snapshot) {
        return Err(Refusal::CurrentUser);
    }

    let admin = in_admin_group(user, None, accounts, changes);

    if admin && admins_unknown(&admins_after(snapshot, accounts, changes, None), accounts) {
        return Err(Refusal::LastAdmin);
    }

    // Without the flags another admin may be locked already.
    if admin && snapshot.part::<AccountLockFlags>().is_none() {
        return Err(Refusal::LockStateUnknown);
    }

    if takes_last_admin(user, None, changes, snapshot) {
        return Err(Refusal::LastAdminLock);
    }

    Ok(())
}

pub(super) fn is_current(user: &UserName, snapshot: &Snapshot) -> bool {
    snapshot
        .part::<CurrentUser>()
        .is_some_and(|current| current.0 == *user)
}

/// The rule behind [`Refusal::LastAdmin`] and [`Refusal::LastAdminLock`]:
/// whether `user` losing the rights (leaving the admin group of `leaving`,
/// or with `None` going or getting a locked password) may leave nobody to
/// administer the system once the queue is applied. True for a member of an
/// admin group when no other admin stays or the admins cannot be told.
pub(super) fn takes_last_admin(
    user: &UserName,
    leaving: Option<&Membership>,
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> bool {
    let Some(accounts) = snapshot.part::<Accounts>() else {
        return true;
    };

    if !in_admin_group(user, leaving, accounts, changes) {
        return false;
    }

    let admins = admins_after(snapshot, accounts, changes, None);

    if admins_unknown(&admins, accounts) {
        return true;
    }

    leaving.map_or_else(
        || admins.iter().all(|admin| admin == user),
        |membership| admins_after(snapshot, accounts, changes, Some(membership)).is_empty(),
    )
}

/// No admin is left to count, or `wheel` (the first of [`ADMIN_GROUPS`],
/// the group Arch's sudoers grants) is missing, so a group only the
/// sudoers file names may be the one that counts.
fn admins_unknown(admins: &BTreeSet<UserName>, accounts: &Accounts) -> bool {
    let [wheel, _] = &*ADMIN_GROUPS;
    admins.is_empty() || accounts.group(wheel).is_none()
}

/// Whether `user` is in the admin group of `only`, or in any of
/// [`ADMIN_GROUPS`], now or by a queued join.
fn in_admin_group(
    user: &UserName,
    only: Option<&Membership>,
    accounts: &Accounts,
    changes: &ChangeSet,
) -> bool {
    ADMIN_GROUPS
        .iter()
        .filter(|group| only.is_none_or(|membership| membership.group == **group))
        .any(|group| {
            accounts.is_member(user, group)
                || changes.wish::<Groups>(&Membership::new(user.clone(), group.clone()))
                    == Some(&Toggle::On)
        })
}

/// The members of [`ADMIN_GROUPS`] who can still log in once the queue is
/// applied: queued memberships counted, users queued for removal and
/// users whose password is locked then left out, and `leaving` counted
/// as left.
fn admins_after(
    snapshot: &Snapshot,
    accounts: &Accounts,
    changes: &ChangeSet,
    leaving: Option<&Membership>,
) -> BTreeSet<UserName> {
    accounts
        .users
        .iter()
        .map(|user| &user.name)
        .filter(|user| changes.wish::<RemovedUsers>(user).is_none())
        .filter(|user| !locked_after(user, changes, snapshot))
        .filter(|user| {
            ADMIN_GROUPS.iter().any(|group| {
                let membership = Membership::new((*user).clone(), group.clone());

                if leaving == Some(&membership) {
                    return false;
                }

                changes
                    .wish::<Groups>(&membership)
                    .map_or_else(|| accounts.is_member(user, group), |wish| wish.is_on())
            })
        })
        .cloned()
        .collect()
}

/// Whether the password of `user` is locked once the queue is applied.
/// Without the helper's flags only a queued lock counts, so removals and
/// leaves are counted by membership alone.
fn locked_after(user: &UserName, changes: &ChangeSet, snapshot: &Snapshot) -> bool {
    changes.wish::<AccountLocks>(user).map_or_else(
        || {
            snapshot
                .part::<AccountLockFlags>()
                .and_then(|flags| flags.is_locked(user))
                == Some(true)
        },
        |wish| wish.is_on(),
    )
}

/// Whether `group` may be removed.
///
/// # Errors
///
/// [`Refusal`] for an unknown group, a system group, or the primary
/// group of a user.
pub(super) fn removable_group(group: &GroupName, accounts: &Accounts) -> Result<(), Refusal> {
    let entry = accounts.group(group).ok_or(Refusal::UnknownGroup)?;

    if entry.is_system() {
        return Err(Refusal::SystemGroup);
    }

    if accounts.is_primary_of_someone(group) {
        return Err(Refusal::PrimaryGroup);
    }

    Ok(())
}
