use std::collections::BTreeSet;

use toolbox_plan::{FullName, GroupName, HomeChange, HomeFolder, ShellPath, UserName};

use super::editable;
use super::guards::is_current;
use crate::domains::{
    AccountLocks, Accounts, FullNames, Groups, HomeFolders, LoginShells, Membership, NewGroups,
    RemovedGroups, RemovedUsers, User,
};
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueuedForUser {
    pub removal: Option<HomeFolder>,
    pub lock: Option<Toggle>,
    pub shell: Option<ShellPath>,
    pub full_name: Option<FullName>,
    pub home: Option<HomeChange>,
    pub joins: BTreeSet<GroupName>,
    pub leaves: BTreeSet<GroupName>,
}

impl QueuedForUser {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserRow<'accounts> {
    pub user: &'accounts User,
    /// The user id as the search finds it, written once with the row.
    pub uid_text: String,
    pub groups: BTreeSet<GroupName>,
    pub current: bool,
    /// Whether the page may change the account at all (not root).
    pub editable: bool,
    pub queued: QueuedForUser,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupRow {
    pub name: GroupName,
    /// `None` for a group that is only queued.
    pub gid: Option<u32>,
    /// The group id as the search finds it, empty for a queued group.
    pub gid_text: String,
    pub members: BTreeSet<UserName>,
    /// Packages own it: it cannot be removed.
    pub system: bool,
    pub queued: Option<Toggle>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupChoice {
    pub group: GroupName,
    pub member: bool,
    /// The primary group, which the user cannot leave.
    pub primary: bool,
    /// `None` for a group that is only queued.
    pub gid: Option<u32>,
    pub members: usize,
}

pub fn user_rows<'accounts>(
    accounts: &'accounts Accounts,
    snapshot: &Snapshot,
    changes: &ChangeSet,
    regular_only: bool,
) -> Vec<UserRow<'accounts>> {
    let memberships = queued_memberships(changes);

    accounts
        .users
        .iter()
        .filter(|user| !regular_only || user.is_regular())
        .map(|user| UserRow {
            user,
            uid_text: user.uid.to_string(),
            groups: accounts.groups_of(&user.name),
            current: is_current(&user.name, snapshot),
            editable: editable(&user.name, accounts).is_ok(),
            queued: queued_for(&user.name, &memberships, changes),
        })
        .collect()
}

pub fn group_rows(accounts: &Accounts, changes: &ChangeSet) -> Vec<GroupRow> {
    let existing = accounts.groups.iter().map(|group| GroupRow {
        name: group.name.clone(),
        gid: Some(group.gid),
        gid_text: group.gid.to_string(),
        members: accounts.members_of(&group.name),
        system: group.is_system(),
        queued: changes
            .wish::<RemovedGroups>(&group.name)
            .map(|_| Toggle::Off),
    });

    let new = queued_new_groups(changes).into_iter().map(|name| GroupRow {
        name,
        gid: None,
        gid_text: String::new(),
        members: BTreeSet::new(),
        system: false,
        queued: Some(Toggle::On),
    });

    let mut rows: Vec<GroupRow> = existing.chain(new).collect();

    rows.sort_by(|left, right| left.name.cmp(&right.name));
    rows
}

pub fn group_choices(
    user: &UserName,
    accounts: &Accounts,
    changes: &ChangeSet,
) -> Vec<GroupChoice> {
    let existing = accounts
        .groups
        .iter()
        .map(|group| &group.name)
        .filter(|group| changes.wish::<RemovedGroups>(group).is_none())
        .cloned();

    let mut choices: Vec<GroupChoice> = existing
        .chain(queued_new_groups(changes))
        .map(|group| {
            let membership = Membership::new(user.clone(), group);
            let member = changes.wish::<Groups>(&membership).map_or_else(
                || accounts.is_member(user, &membership.group),
                |wish| wish.is_on(),
            );

            GroupChoice {
                primary: accounts.is_primary(user, &membership.group),
                gid: accounts.group(&membership.group).map(|entry| entry.gid),
                members: accounts.members_of(&membership.group).len(),
                group: membership.group,
                member,
            }
        })
        .collect();

    choices.sort_by(|left, right| left.group.cmp(&right.group));
    choices
}

fn queued_new_groups(changes: &ChangeSet) -> Vec<GroupName> {
    changes
        .queued::<NewGroups>()
        .into_iter()
        .map(|(group, _)| group)
        .collect()
}

pub fn queued_memberships(changes: &ChangeSet) -> Vec<(Membership, bool)> {
    changes
        .queued::<Groups>()
        .into_iter()
        .map(|(membership, wish)| (membership, wish.is_on()))
        .collect()
}

fn queued_for(
    user: &UserName,
    memberships: &[(Membership, bool)],
    changes: &ChangeSet,
) -> QueuedForUser {
    let groups = |joining: bool| -> BTreeSet<GroupName> {
        memberships
            .iter()
            .filter(|(membership, on)| membership.user == *user && *on == joining)
            .map(|(membership, _)| membership.group.clone())
            .collect()
    };

    QueuedForUser {
        removal: changes.wish::<RemovedUsers>(user).copied(),
        lock: changes.wish::<AccountLocks>(user).copied(),
        shell: changes.wish::<LoginShells>(user).cloned(),
        full_name: changes.wish::<FullNames>(user).cloned(),
        home: changes.wish::<HomeFolders>(user).cloned(),
        joins: groups(true),
        leaves: groups(false),
    }
}
