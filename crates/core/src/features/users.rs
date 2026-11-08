//! The Users and groups page: what may be changed, and how each click
//! becomes a queued change. Changes that carry a password do not queue,
//! they become a plan that runs at once (`now`).

mod guards;
pub mod home;
pub mod input;
pub mod listing;
pub mod now;
mod recheck;
pub mod rows;
pub mod run;
#[cfg(test)]
mod tests;
pub mod view;

use std::collections::BTreeSet;
use std::sync::LazyLock;

use guards::{lockable, removable, removable_group, takes_last_admin};
pub use recheck::{recheck, recheck_groups, recheck_locks};
use toolbox_plan::{FullName, GroupName, HomeFolder, ShellPath, UserName};

use crate::domains::{
    AccountLocks, Accounts, AvailableShells, FullNames, Groups, HomeFolders, LoginShells,
    Membership, NewGroups, RemovedGroups, RemovedUsers, User,
};
use crate::{ChangeSet, Queued, Snapshot, Toggle};

/// The groups of [`toolbox_plan::ADMIN_GROUPS`]. A system whose last
/// member of all of them left could no longer be administered.
#[expect(clippy::expect_used, reason = "constant names, parsed by a test")]
pub static ADMIN_GROUPS: LazyLock<[GroupName; 2]> = LazyLock::new(|| {
    toolbox_plan::ADMIN_GROUPS.map(|name| GroupName::new(name).expect("a constant group name"))
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    /// The account files could not be read.
    Unavailable,
    UnknownUser,
    UnknownGroup,
    /// Accounts with user id 0 are not changed here at all.
    Root,
    /// The account the app runs for cannot remove or lock itself.
    CurrentUser,
    /// Only accounts of people are removed.
    SystemAccount,
    /// The last member of [`ADMIN_GROUPS`] keeps one of them.
    LastAdmin,
    /// The last member of [`ADMIN_GROUPS`] keeps a password to use them.
    LastAdminLock,
    /// An admin keeps a password to use the rights while it is unknown
    /// which other admins are locked.
    LockStateUnknown,
    /// A user cannot leave its primary group, and a group that is the
    /// primary group of a user cannot be removed.
    PrimaryGroup,
    SystemGroup,
    UserExists,
    GroupExists,
    /// `/etc/shells` does not list the shell.
    ShellNotListed,
    /// An account already has the user id.
    UidInUse,
    /// The user id lies outside the range of the kind of account.
    UidOutOfRange,
    /// Another account lives in the folder.
    HomeTaken,
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// The account files of `snapshot`.
///
/// # Errors
///
/// [`Refusal::Unavailable`] while no probe delivered them.
pub fn accounts(snapshot: &Snapshot) -> Result<&Accounts, Refusal> {
    snapshot.part::<Accounts>().ok_or(Refusal::Unavailable)
}

/// The account `name`, when the page may change it.
///
/// # Errors
///
/// [`Refusal::UnknownUser`] or [`Refusal::Root`].
pub fn editable<'accounts>(
    name: &UserName,
    accounts: &'accounts Accounts,
) -> Result<&'accounts User, Refusal> {
    let user = accounts.user(name).ok_or(Refusal::UnknownUser)?;

    if user.is_root() {
        Err(Refusal::Root)
    } else {
        Ok(user)
    }
}

/// Whether one membership may get the state `wish`.
///
/// A catalog toggle changes one group where the page's dialog sets all of
/// them, and meets the same guards. The group may not exist yet: the
/// package of a queued toggle creates it (`docker`).
///
/// # Errors
///
/// [`Refusal`] for root, an unknown user, the user's primary group, or
/// when the last admin would leave [`ADMIN_GROUPS`].
pub fn membership_allowed(
    membership: &Membership,
    wish: Toggle,
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    editable(&membership.user, accounts)?;

    if accounts.is_primary(&membership.user, &membership.group) {
        return Err(Refusal::PrimaryGroup);
    }

    if !wish.is_on() && takes_last_admin(&membership.user, Some(membership), changes, snapshot) {
        return Err(Refusal::LastAdmin);
    }

    Ok(())
}

/// Queues the groups of `user` to become exactly `wanted` (beside its
/// primary group, which it cannot leave). Nothing changes when a guard
/// refuses.
///
/// # Errors
///
/// [`Refusal`] for root, an unknown user or group, or when the last
/// member of [`ADMIN_GROUPS`] would leave the last of them.
pub fn set_groups(
    user: &UserName,
    wanted: &BTreeSet<GroupName>,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    editable(user, accounts)?;

    if wanted
        .iter()
        .any(|group| !group_exists_after(group, accounts, changes))
    {
        return Err(Refusal::UnknownGroup);
    }

    if !wanted.iter().any(GroupName::is_admin) && takes_last_admin(user, None, changes, snapshot) {
        return Err(Refusal::LastAdmin);
    }

    let known = accounts.groups.iter().map(|group| &group.name);

    for group in known.chain(wanted) {
        if accounts.is_primary(user, group) {
            continue;
        }

        changes.want::<Groups>(
            Membership::new(user.clone(), group.clone()),
            Toggle::from_on(wanted.contains(group)),
            snapshot,
        );
    }

    Ok(())
}

/// Queues locking or unlocking the password of `user`.
///
/// A second click on the same wish takes it back, and a wish the system already
/// fulfils (`known` is the flag the helper read, when it could) queues nothing.
///
/// # Errors
///
/// [`Refusal`] for root, an unknown user, locking the app's own user, or
/// locking a member of [`ADMIN_GROUPS`] while the admins cannot be told.
pub fn set_locked(
    user: &UserName,
    wish: Toggle,
    known: Option<bool>,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    lockable(user, wish, changes, snapshot)?;

    if changes.wish::<AccountLocks>(user) == Some(&wish) || known == Some(wish.is_on()) {
        changes.remove::<AccountLocks>(user);
    } else {
        changes.want::<AccountLocks>(user.clone(), wish, snapshot);
    }

    Ok(())
}

/// Queues the removal of `user` and drops everything else queued for the
/// account: it would change an account that is about to go.
///
/// # Errors
///
/// [`Refusal`] for root, the app's own user, a system account, an unknown
/// user or the last member of [`ADMIN_GROUPS`].
pub fn remove_user(
    user: &UserName,
    home: HomeFolder,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    removable(user, changes, snapshot)?;
    forget_user(user, changes);
    changes.want::<RemovedUsers>(user.clone(), home, snapshot);
    Ok(())
}

pub fn keep_user(user: &UserName, changes: &mut ChangeSet) {
    changes.remove::<RemovedUsers>(user);
}

/// Queues a new group, or takes the queued removal of an existing one
/// back.
///
/// # Errors
///
/// [`Refusal::GroupExists`] for a group that exists and stays.
pub fn add_group(
    group: &GroupName,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    if changes.wish::<RemovedGroups>(group).is_some() {
        changes.remove::<RemovedGroups>(group);
        return Ok(());
    }

    if accounts.group(group).is_some() || changes.wish::<NewGroups>(group).is_some() {
        return Err(Refusal::GroupExists);
    }

    changes.want::<NewGroups>(group.clone(), Queued, snapshot);
    Ok(())
}

/// Queues the removal of `group`, or takes a queued new group back. The
/// memberships queued for it go too: gpasswd would fail on a group that
/// does not exist.
///
/// # Errors
///
/// [`Refusal`] for an unknown group, a system group, or the primary
/// group of a user.
pub fn remove_group(
    group: &GroupName,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    if changes.wish::<NewGroups>(group).is_some() {
        changes.remove::<NewGroups>(group);
        forget_group(group, changes);
        return Ok(());
    }

    removable_group(group, accounts)?;
    forget_group(group, changes);
    changes.want::<RemovedGroups>(group.clone(), Queued, snapshot);
    Ok(())
}

/// Queues `shell` as the login shell of `user`.
///
/// # Errors
///
/// [`Refusal`] for root, an unknown user, or a shell `/etc/shells` does
/// not list (the helper would refuse it too).
pub fn set_shell(
    user: &UserName,
    shell: ShellPath,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    editable(user, accounts(snapshot)?)?;

    if !shell_is_listed(&shell, snapshot) {
        return Err(Refusal::ShellNotListed);
    }

    changes.want::<LoginShells>(user.clone(), shell, snapshot);
    Ok(())
}

/// Queues `full_name` as the full name of `user`.
///
/// # Errors
///
/// [`Refusal`] for root or an unknown user.
pub fn set_full_name(
    user: &UserName,
    full_name: FullName,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    editable(user, accounts(snapshot)?)?;
    changes.want::<FullNames>(user.clone(), full_name, snapshot);
    Ok(())
}

fn shell_is_listed(shell: &ShellPath, snapshot: &Snapshot) -> bool {
    snapshot
        .part::<AvailableShells>()
        .is_some_and(|shells| shells.0.contains(shell))
}

/// Whether `group` exists once the queue is applied.
fn group_exists_after(group: &GroupName, accounts: &Accounts, changes: &ChangeSet) -> bool {
    if changes.wish::<RemovedGroups>(group).is_some() {
        return false;
    }

    accounts.group(group).is_some() || changes.wish::<NewGroups>(group).is_some()
}

fn forget_user(user: &UserName, changes: &mut ChangeSet) {
    for (membership, _) in rows::queued_memberships(changes) {
        if membership.user == *user {
            changes.remove::<Groups>(&membership);
        }
    }

    changes.remove::<LoginShells>(user);
    changes.remove::<FullNames>(user);
    changes.remove::<HomeFolders>(user);
    changes.remove::<AccountLocks>(user);
}

fn forget_group(group: &GroupName, changes: &mut ChangeSet) {
    for (membership, _) in rows::queued_memberships(changes) {
        if membership.group == *group {
            changes.remove::<Groups>(&membership);
        }
    }
}
