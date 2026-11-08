//! The users and groups of `/etc/passwd` and `/etc/group`, as one part of
//! the snapshot. The account changes all compare with it.

use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::{GroupName, UserName};

/// `UID_MIN` and `UID_MAX` of `/etc/login.defs`: the ids useradd gives to
/// people. Below are system accounts, above `nobody` and container ranges.
pub const FIRST_REGULAR_ID: u32 = 1000;
pub const LAST_REGULAR_ID: u32 = 60_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub name: UserName,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub home: String,
    /// As the file has it, which may be empty or a program our rules for
    /// login shells reject.
    pub shell: String,
}

impl User {
    pub fn full_name(&self) -> &str {
        self.gecos.split(',').next().unwrap_or_default()
    }

    pub const fn is_regular(&self) -> bool {
        self.uid >= FIRST_REGULAR_ID && self.uid <= LAST_REGULAR_ID
    }

    /// Every account with user id 0 is root, whatever it is called.
    pub const fn is_root(&self) -> bool {
        self.uid == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub name: GroupName,
    pub gid: u32,
    /// The members the entry lists. Users with this primary group are
    /// members too without being listed.
    pub members: BTreeSet<UserName>,
}

impl Group {
    pub const fn is_system(&self) -> bool {
        self.gid < FIRST_REGULAR_ID
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Accounts {
    pub users: Vec<User>,
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentUser(pub UserName);

/// Whether the password of each account is locked.
///
/// The root helper reads it from `/etc/shadow`, which no probe may read.
/// So this part arrives later than the others, once the Users page asked
/// the helper, and is missing while the helper could not be asked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountLockFlags(pub BTreeMap<UserName, bool>);

impl AccountLockFlags {
    /// `None` for an account the answer does not name.
    pub fn is_locked(&self, user: &UserName) -> Option<bool> {
        self.0.get(user).copied()
    }
}

impl Accounts {
    pub fn user(&self, name: &UserName) -> Option<&User> {
        self.users.iter().find(|user| user.name == *name)
    }

    pub fn group(&self, name: &GroupName) -> Option<&Group> {
        self.groups.iter().find(|group| group.name == *name)
    }

    pub fn is_primary(&self, user: &UserName, group: &GroupName) -> bool {
        match (self.user(user), self.group(group)) {
            (Some(user), Some(group)) => user.gid == group.gid,
            (None, _) | (_, None) => false,
        }
    }

    /// Listed in the group or having it as primary group, as `id -Gn`
    /// counts.
    pub fn is_member(&self, user: &UserName, group: &GroupName) -> bool {
        self.group(group)
            .is_some_and(|entry| entry.members.contains(user))
            || self.is_primary(user, group)
    }

    pub fn groups_of(&self, user: &UserName) -> BTreeSet<GroupName> {
        self.groups
            .iter()
            .filter(|group| self.is_member(user, &group.name))
            .map(|group| group.name.clone())
            .collect()
    }

    pub fn members_of(&self, group: &GroupName) -> BTreeSet<UserName> {
        self.users
            .iter()
            .filter(|user| self.is_member(&user.name, group))
            .map(|user| user.name.clone())
            .chain(
                self.group(group)
                    .into_iter()
                    .flat_map(|entry| entry.members.iter().cloned()),
            )
            .collect()
    }

    /// Whether some user has `group` as primary group. Such a group
    /// cannot be removed.
    pub fn is_primary_of_someone(&self, group: &GroupName) -> bool {
        self.group(group)
            .is_some_and(|entry| self.users.iter().any(|user| user.gid == entry.gid))
    }
}
