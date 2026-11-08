//! The queued account changes of the Users page besides memberships and
//! shells: locks, full names, home folders, removed users, new and removed
//! groups. All go through the root helper in one apply. Changes that
//! carry a password never queue: they run at once (`features::users`).

#[cfg(test)]
mod tests;

use toolbox_plan::{FullName, GroupName, HomeChange, HomeFolder, RootAction, UserName};

use super::accounts::{self, Accounts};
use super::each;
use crate::Snapshot;
use crate::domain::{Domain, Queued, Stage, Staged, Summary, Toggle};

/// Before the memberships, so one apply can add a group and fill it.
pub struct NewGroups;

/// `On` locks the password, `Off` unlocks it. The lock flags live in
/// `/etc/shadow`, which no probe of the snapshot may read, so this is
/// one-off work that leaves the queue after a successful apply.
pub struct AccountLocks;

pub struct FullNames;

/// A new home folder for an existing account, with or without its files.
pub struct HomeFolders;

/// After everything that changes an account, so a queued change of a user
/// that is removed in the same apply still finds the account.
pub struct RemovedUsers;

/// Last: a group can only go once no user has it as primary group, and
/// userdel removes a user's own group itself.
pub struct RemovedGroups;

impl Domain for NewGroups {
    type Item = GroupName;
    type State = Accounts;
    type Wish = Queued;

    const ID: &'static str = "new-groups";
    const STAGE: Stage = accounts::NEW_GROUPS;

    fn is_satisfied(item: &GroupName, _wish: &Queued, state: &Accounts) -> bool {
        state.group(item).is_some()
    }

    fn plan(changes: &[(&GroupName, &Queued)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |group, _| RootAction::CreateGroup {
            group: group.clone(),
        })
    }

    named_key!();

    fn summary(item: &GroupName, _wish: &Queued) -> Summary {
        Summary::of("group-add", item.as_str())
    }
}

impl Domain for AccountLocks {
    type Item = UserName;
    type State = ();
    type Wish = Toggle;

    const ID: &'static str = "account-locks";
    const ONE_OFF: bool = true;
    const STAGE: Stage = accounts::LOCKS;

    fn state(_snapshot: &Snapshot) -> Option<&()> {
        Some(&())
    }

    /// Both wishes are work to do. Taking one back removes it.
    fn is_satisfied(_item: &UserName, _wish: &Toggle, (): &()) -> bool {
        false
    }

    fn plan(changes: &[(&UserName, &Toggle)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |user, wish| {
            RootAction::SetUserLocked {
                user: user.clone(),
                locked: wish.is_on(),
            }
        })
    }

    named_key!();

    fn summary(item: &UserName, wish: &Toggle) -> Summary {
        Summary::toggle(*wish, "account-lock", "account-unlock", item.as_str())
    }
}

impl Domain for FullNames {
    type Item = UserName;
    type State = Accounts;
    type Wish = FullName;

    const ID: &'static str = "full-names";
    const STAGE: Stage = accounts::FULL_NAMES;

    fn is_satisfied(item: &UserName, wish: &FullName, state: &Accounts) -> bool {
        state
            .user(item)
            .is_some_and(|user| user.full_name() == wish.as_str())
    }

    fn plan(changes: &[(&UserName, &FullName)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |user, full_name| {
            RootAction::SetFullName {
                user: user.clone(),
                full_name: full_name.clone(),
            }
        })
    }

    named_key!();

    fn summary(item: &UserName, wish: &FullName) -> Summary {
        if wish.is_empty() {
            Summary::of("full-name-clear", item.as_str())
        } else {
            Summary::pair("full-name-set", item.as_str(), wish.as_str())
        }
    }
}

impl Domain for HomeFolders {
    type Item = UserName;
    type State = Accounts;
    type Wish = HomeChange;

    const ID: &'static str = "home-folders";
    const STAGE: Stage = accounts::HOME_FOLDERS;

    fn is_satisfied(item: &UserName, wish: &HomeChange, state: &Accounts) -> bool {
        state
            .user(item)
            .is_some_and(|user| user.home == wish.home.as_str())
    }

    fn plan(changes: &[(&UserName, &HomeChange)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |user, change| {
            RootAction::SetHomeFolder {
                user: user.clone(),
                change: change.clone(),
            }
        })
    }

    named_key!();

    fn summary(item: &UserName, wish: &HomeChange) -> Summary {
        Summary::pair("home-folder", item.as_str(), wish.home.as_str())
    }
}

impl Domain for RemovedUsers {
    type Item = UserName;
    type State = Accounts;
    type Wish = HomeFolder;

    const ID: &'static str = "removed-users";
    const STAGE: Stage = accounts::REMOVED_USERS;

    fn is_satisfied(item: &UserName, _wish: &HomeFolder, state: &Accounts) -> bool {
        state.user(item).is_none()
    }

    fn plan(changes: &[(&UserName, &HomeFolder)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |user, home| RootAction::DeleteUser {
            user: user.clone(),
            home: *home,
        })
    }

    named_key!();

    fn summary(item: &UserName, wish: &HomeFolder) -> Summary {
        let code = match wish {
            HomeFolder::Delete => "user-remove-with-home",
            HomeFolder::Keep => "user-remove-keep-home",
        };

        Summary::of(code, item.as_str())
    }
}

impl Domain for RemovedGroups {
    type Item = GroupName;
    type State = Accounts;
    type Wish = Queued;

    const ID: &'static str = "removed-groups";
    const STAGE: Stage = accounts::REMOVED_GROUPS;

    fn is_satisfied(item: &GroupName, _wish: &Queued, state: &Accounts) -> bool {
        state.group(item).is_none()
    }

    fn plan(changes: &[(&GroupName, &Queued)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |group, _| RootAction::DeleteGroup {
            group: group.clone(),
        })
    }

    named_key!();

    fn summary(item: &GroupName, _wish: &Queued) -> Summary {
        Summary::of("group-remove", item.as_str())
    }
}
