pub mod boot;
pub mod bootrepair;
pub mod configs;
pub mod console;
pub mod diagnostics;
pub mod discover;
pub mod kernels;
pub mod live;
pub mod locales;
pub mod maintenance;
pub mod missing;
pub mod network;
pub mod packages;
pub mod recent;
pub mod search;
pub mod services;
pub mod status;
#[cfg(test)]
mod tests;
pub mod toggles;
pub mod users;

use std::collections::BTreeSet;

use toolbox_plan::Reason;

use crate::{ChangeSet, Domain, Snapshot};

/// Every guard by its id, in the order they run. A new page that queues
/// changes adds its line here.
pub const GUARDS: &[(&str, Recheck)] = &[
    ("users", users::recheck),
    ("memberships", users::recheck_groups),
    // Apart from the users: the lock flags come from the helper once the
    // Users page asks, long after the account files.
    ("user-locks", users::recheck_locks),
    ("kernels", kernels::recheck),
    ("locales", locales::recheck),
    ("language-packs", locales::view::drop_stranded_packs),
    ("plymouth", boot::recheck),
    ("grub", boot::recheck_grub),
    ("module-blacklist", network::recheck_blacklist),
    ("new-groups", missing::recheck_new_groups),
    ("hblock", missing::recheck_hblock),
    ("wifi-backend", missing::recheck_wifi_backend),
    // Waits for the AUR's copy, which the window reads only on demand.
    ("aur-packages", packages::aur::recheck),
];

/// One page's check of a queue restored from disk: how many changes it
/// dropped, or `None` while the snapshot lacks the part it checks
/// against.
pub type Recheck = fn(&mut ChangeSet, &Snapshot) -> Option<usize>;

/// The guards a queue restored from disk has not met yet. It was queued
/// again before the system was known, so each guard runs once, with the
/// first snapshot that holds what it checks against.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Unchecked {
    waiting: BTreeSet<&'static str>,
}

impl Unchecked {
    pub fn all() -> Self {
        Self {
            waiting: GUARDS.iter().map(|(id, _)| *id).collect(),
        }
    }

    /// Runs the waiting guards `snapshot` has the parts for. Returns how
    /// many changes they dropped, each with a line in the log.
    pub fn recheck(&mut self, changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
        let mut dropped = 0;

        for &(id, run) in GUARDS {
            if !self.waiting.contains(id) {
                continue;
            }

            if let Some(count) = run(changes, snapshot) {
                self.waiting.remove(id);
                dropped += count;
            }
        }

        dropped
    }
}

/// Drops every queued change of `D` that `refusal` gives a reason for,
/// each with a line in the log. Returns how many went.
fn drop_refused<D>(
    changes: &mut ChangeSet,
    what: &str,
    mut refusal: impl FnMut(&D::Item, &D::Wish, &ChangeSet) -> Option<&'static str>,
) -> usize
where
    D: Domain,
{
    let mut dropped = 0;

    for (item, wish) in changes.queued::<D>() {
        let Some(why) = refusal(&item, &wish, changes) else {
            continue;
        };

        changes.remove::<D>(&item);
        tracing::warn!("dropped the restored {what} {}: {why}", D::key(&item));
        dropped += 1;
    }

    dropped
}

pub const fn reason_code(reason: Reason) -> &'static str {
    match reason {
        Reason::Empty => "empty",
        Reason::TooLong { max: _ } => "too-long",
        Reason::InvalidStart(_) => "invalid-start",
        Reason::InvalidCharacter(_) => "invalid-character",
        Reason::InvalidComponent => "invalid-path",
        Reason::WrongFileType => "wrong-type",
        Reason::UnknownUnitType | Reason::UnknownDevice => "invalid",
    }
}
