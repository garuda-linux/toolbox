//! The search and the order of the two lists.

use std::cmp::Ordering;
use std::iter;

use toolbox_plan::{GroupName, UserName};

use super::rows::{GroupRow, UserRow};
use crate::features::search::{Needle, Searchable, folded};

/// What a list is ordered by. Ties fall back to the name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum UserOrder {
    #[default]
    Name,
    Id,
    FullName,
    Home,
    Shell,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum GroupOrder {
    #[default]
    Name,
    Id,
    Members,
}

impl UserOrder {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl GroupOrder {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// A text as the list sorts it: empty texts last, the rest without
/// regard to case.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TextKey {
    empty: bool,
    folded: String,
}

impl TextKey {
    fn of(text: &str) -> Self {
        Self {
            empty: text.is_empty(),
            folded: folded(text),
        }
    }
}

impl Searchable for UserRow<'_> {
    fn texts(&self) -> impl Iterator<Item = &str> {
        let user = self.user;

        [
            user.name.as_str(),
            self.uid_text.as_str(),
            user.full_name(),
            user.home.as_str(),
            user.shell.as_str(),
        ]
        .into_iter()
        .chain(self.groups.iter().map(GroupName::as_str))
    }
}

impl Searchable for GroupRow {
    fn texts(&self) -> impl Iterator<Item = &str> {
        [self.name.as_str(), self.gid_text.as_str()]
            .into_iter()
            .chain(self.members.iter().map(UserName::as_str))
    }
}

/// The group picker finds a group by its name alone.
impl Searchable for GroupName {
    fn texts(&self) -> impl Iterator<Item = &str> {
        iter::once(self.as_str())
    }
}

/// Keeps the users with `search` in their name, user id, full name, home
/// folder, shell or a group name (any case), ordered by `order`.
pub fn arrange_users(rows: &mut Vec<UserRow<'_>>, search: &str, order: UserOrder) {
    let needle = Needle::new(search);

    rows.retain(|row| needle.finds(row));

    rows.sort_by_cached_key(|row| {
        let user = row.user;
        let text = match order {
            UserOrder::Name | UserOrder::Id => None,
            UserOrder::FullName => Some(user.full_name()),
            UserOrder::Home => Some(user.home.as_str()),
            UserOrder::Shell => Some(user.shell.as_str()),
        };

        let id = (order == UserOrder::Id).then_some(user.uid);

        (text.map(TextKey::of), id, &user.name)
    });
}

/// Keeps the groups with `search` in their name, group id or a member's
/// name (any case), ordered by `order`. A queued group has no id yet and
/// comes last by id.
pub fn arrange_groups(rows: &mut Vec<GroupRow>, search: &str, order: GroupOrder) {
    let needle = Needle::new(search);

    rows.retain(|row| needle.finds(row));
    rows.sort_by(|left, right| {
        let first = match order {
            GroupOrder::Name => Ordering::Equal,
            GroupOrder::Id => left
                .gid
                .is_none()
                .cmp(&right.gid.is_none())
                .then_with(|| left.gid.cmp(&right.gid)),
            GroupOrder::Members => right.members.len().cmp(&left.members.len()),
        };

        first.then_with(|| left.name.cmp(&right.name))
    });
}
