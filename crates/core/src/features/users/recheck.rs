//! The guards for a queue that came from disk. It was restored before
//! the system was known, so nothing checked it.

use super::guards::{is_current, lockable, removable, removable_group, takes_last_admin};
use super::home::is_taken;
use super::{Accounts, Refusal, accounts, editable, membership_allowed, shell_is_listed};
use crate::domains::{
    AccountLockFlags, AccountLocks, CurrentUser, FullNames, Groups, HomeFolders, LoginShells,
    RemovedGroups, RemovedUsers,
};
use crate::features::drop_refused;
use crate::{ChangeSet, Queued, Snapshot, Toggle};

/// Drops every queued removal, leave and account detail the page would
/// refuse. The locks wait for their flags in [`recheck_locks`].
///
/// Returns how many went, `None` without the account files or the current
/// user: nothing can be checked then and everything stays.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let accounts = accounts(snapshot).ok()?;
    snapshot.part::<CurrentUser>()?;

    Some(
        removals(changes, snapshot)
            + admin_leaves(changes, snapshot)
            + texts(changes, snapshot, accounts),
    )
}

fn removals(changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
    let mut dropped = 0;

    for (user, home) in changes.queued::<RemovedUsers>() {
        // Its own removal would hide the user from the count of admins.
        changes.remove::<RemovedUsers>(&user);

        match removable(&user, changes, snapshot) {
            Ok(()) => changes.want::<RemovedUsers>(user, home, snapshot),
            Err(refusal) => {
                warn("removal of", user.as_str(), refusal);
                dropped += 1;
            }
        }
    }

    dropped
}

/// Drops the removals and admin-group leaves the lock flags now refuse,
/// then the queued locks the page would refuse.
///
/// Returns how many went, `None` without the account files, the current
/// user or the lock flags: nothing can be checked then and all stays.
pub fn recheck_locks(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    accounts(snapshot).ok()?;
    snapshot.part::<CurrentUser>()?;
    snapshot.part::<AccountLockFlags>()?;

    // A lock is easily undone and a removal is not, so where they compete
    // for the last unlocked admin the removal is the one dropped.
    Some(removals(changes, snapshot) + admin_leaves(changes, snapshot) + locks(changes, snapshot))
}

/// Drops every queued membership and group removal the page would
/// refuse: root, an unknown user, a primary group, a system group.
///
/// Returns how many went, `None` without the account files.
pub fn recheck_groups(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let accounts = accounts(snapshot).ok()?;

    Some(memberships(changes, snapshot) + removed_groups(changes, accounts))
}

fn memberships(changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
    let mut dropped = 0;

    for (membership, wish) in changes.queued::<Groups>() {
        // Counted as queued, a leave of an admin group takes itself for
        // the last admin's.
        changes.remove::<Groups>(&membership);

        match membership_allowed(&membership, wish, changes, snapshot) {
            Ok(()) => changes.want::<Groups>(membership, wish, snapshot),
            Err(refusal) => {
                warn("membership of", membership.user.as_str(), refusal);
                dropped += 1;
            }
        }
    }

    dropped
}

fn removed_groups(changes: &mut ChangeSet, accounts: &Accounts) -> usize {
    drop_refused::<RemovedGroups>(changes, "removal of the group", |group, Queued, _| {
        removable_group(group, accounts).err().map(Refusal::as_str)
    })
}

fn locks(changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
    drop_refused::<AccountLocks>(changes, "lock change of", |user, wish, changes| {
        lockable(user, *wish, changes, snapshot)
            .err()
            .map(Refusal::as_str)
    })
}

fn admin_leaves(changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
    let mut dropped = 0;

    for (membership, wish) in changes.queued::<Groups>() {
        if !membership.group.is_admin() || wish.is_on() {
            continue;
        }

        // Counted as a member again, the user shows as the last admin.
        changes.remove::<Groups>(&membership);

        if takes_last_admin(&membership.user, Some(&membership), changes, snapshot) {
            warn(
                "leave of an admin group by",
                membership.user.as_str(),
                Refusal::LastAdmin,
            );
            dropped += 1;
        } else {
            changes.want::<Groups>(membership, Toggle::Off, snapshot);
        }
    }

    dropped
}

fn warn(what: &str, user: &str, refusal: Refusal) {
    tracing::warn!("dropped the restored {what} {user}: {}", refusal.as_str());
}

/// Drops the account details the page would refuse: root, an unknown user,
/// a shell `/etc/shells` does not list, the app's own user's home folder
/// and a home folder another account has.
fn texts(changes: &mut ChangeSet, snapshot: &Snapshot, accounts: &Accounts) -> usize {
    let shells = drop_refused::<LoginShells>(changes, "shell change of", |user, shell, _| {
        let refusal = match editable(user, accounts) {
            Err(refusal) => refusal,
            Ok(_) if shell_is_listed(shell, snapshot) => return None,
            Ok(_) => Refusal::ShellNotListed,
        };

        Some(refusal.as_str())
    });

    let names = drop_refused::<FullNames>(changes, "full name change of", |user, _, _| {
        editable(user, accounts).err().map(Refusal::as_str)
    });

    let homes = drop_refused::<HomeFolders>(changes, "home folder change of", |user, change, _| {
        let refusal = match editable(user, accounts) {
            Err(refusal) => refusal,
            Ok(_) if is_current(user, snapshot) => Refusal::CurrentUser,
            Ok(_) if is_taken(&change.home, Some(user), accounts) => Refusal::HomeTaken,
            Ok(_) => return None,
        };

        Some(refusal.as_str())
    });

    shells + names + homes
}
