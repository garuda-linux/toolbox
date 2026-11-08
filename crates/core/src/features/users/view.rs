//! The window and the CLI draw from here.

#[cfg(test)]
mod tests;

use toolbox_plan::{GroupName, ShellPath, UserName};

use super::ADMIN_GROUPS;
use super::listing::{GroupOrder, UserOrder, arrange_groups, arrange_users};
use super::rows::{
    GroupChoice, GroupRow, QueuedForUser, UserRow, group_choices, group_rows, user_rows,
};
use crate::domains::{Accounts, AvailableShells};
use crate::features::search::Needle;
use crate::{ChangeSet, Snapshot, Toggle};

/// What the lock entry of a user's menu does now: a queued lock or unlock
/// is taken back, else the account goes the other way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockAction {
    Lock,
    Unlock,
    UnqueueLock,
    UnqueueUnlock,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    pub regular_only: bool,
    pub user_search: String,
    pub user_order: UserOrder,
    pub group_search: String,
    pub group_order: GroupOrder,
    pub new_group_search: String,
}

impl Shown {
    pub fn people() -> Self {
        Self {
            regular_only: true,
            ..Self::default()
        }
    }
}

impl LockAction {
    /// The entry for a user with `queued` lock wish and `locked` state.
    /// `None` while the state is not known and nothing is queued.
    pub const fn of(queued: Option<Toggle>, locked: Option<bool>) -> Option<Self> {
        match (queued, locked) {
            (Some(Toggle::On), _) => Some(Self::UnqueueLock),
            (Some(Toggle::Off), _) => Some(Self::UnqueueUnlock),
            (None, Some(true)) => Some(Self::Unlock),
            (None, Some(false)) => Some(Self::Lock),
            (None, None) => None,
        }
    }

    /// Whether the entry asks for a lock, the `wish` of [`super::set_locked`]:
    /// the same wish again takes a queued one back.
    pub const fn locked(self) -> bool {
        matches!(self, Self::Lock | Self::UnqueueLock)
    }
}

impl QueuedForUser {
    pub fn count(&self) -> usize {
        usize::from(self.removal.is_some())
            + usize::from(self.lock.is_some())
            + usize::from(self.shell.is_some())
            + usize::from(self.full_name.is_some())
            + usize::from(self.home.is_some())
            + self.joins.len()
            + self.leaves.len()
    }
}

impl UserRow<'_> {
    pub fn shown_full_name(&self) -> &str {
        self.queued
            .full_name
            .as_ref()
            .map_or_else(|| self.user.full_name(), |name| name.as_str())
    }

    pub fn shown_shell(&self) -> &str {
        self.queued
            .shell
            .as_ref()
            .map_or(self.user.shell.as_str(), ShellPath::as_str)
    }

    pub fn shown_home(&self) -> &str {
        self.queued
            .home
            .as_ref()
            .map_or(self.user.home.as_str(), |change| change.home.as_str())
    }
}

pub fn accounts_known(snapshot: &Snapshot) -> bool {
    snapshot.part::<Accounts>().is_some()
}

pub fn users_total(snapshot: &Snapshot) -> usize {
    snapshot
        .part::<Accounts>()
        .map_or(0, |accounts| accounts.users.len())
}

pub fn users<'snapshot>(
    snapshot: &'snapshot Snapshot,
    changes: &ChangeSet,
    shown: &Shown,
) -> Vec<UserRow<'snapshot>> {
    let Some(accounts) = snapshot.part::<Accounts>() else {
        return Vec::new();
    };

    let mut rows = user_rows(accounts, snapshot, changes, shown.regular_only);
    arrange_users(&mut rows, &shown.user_search, shown.user_order);
    rows
}

pub fn groups(snapshot: &Snapshot, changes: &ChangeSet, shown: &Shown) -> Vec<GroupRow> {
    let Some(accounts) = snapshot.part::<Accounts>() else {
        return Vec::new();
    };

    let mut rows = group_rows(accounts, changes);
    arrange_groups(&mut rows, &shown.group_search, shown.group_order);
    rows
}

pub fn shells(snapshot: &Snapshot) -> &[ShellPath] {
    snapshot
        .part::<AvailableShells>()
        .map_or(&[][..], |shells| shells.0.as_slice())
}

/// The names the add-user dialog offers: every group but one that is
/// queued to be added, which does not exist yet. Only names holding
/// `search`, in any case.
pub fn group_names(snapshot: &Snapshot, changes: &ChangeSet, search: &str) -> Vec<GroupName> {
    let Some(accounts) = snapshot.part::<Accounts>() else {
        return Vec::new();
    };

    let needle = Needle::new(search);
    group_rows(accounts, changes)
        .into_iter()
        .filter(|row| row.queued != Some(Toggle::On))
        .map(|row| row.name)
        .filter(|name| needle.finds(name))
        .collect()
}

/// The group the add-user dialog's "Administrator" switch adds: the
/// first admin group, which Arch's sudoers and polkit rules name. `None`
/// when a new user cannot join it.
pub fn admin_group(snapshot: &Snapshot, changes: &ChangeSet) -> Option<&'static GroupName> {
    let [admin, _] = &*ADMIN_GROUPS;

    group_names(snapshot, changes, "")
        .contains(admin)
        .then_some(admin)
}

pub fn choices(user: &UserName, snapshot: &Snapshot, changes: &ChangeSet) -> Vec<GroupChoice> {
    snapshot
        .part::<Accounts>()
        .map(|accounts| group_choices(user, accounts, changes))
        .unwrap_or_default()
}
