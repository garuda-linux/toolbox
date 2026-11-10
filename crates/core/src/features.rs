pub mod boot;
pub mod bootrepair;
pub mod configs;
pub mod console;
pub mod diagnostics;
pub mod discover;
mod guard;
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
pub mod tweaks;
pub mod users;

use std::collections::BTreeSet;

use strum::IntoEnumIterator as _;
use toolbox_plan::Reason;

pub use self::guard::Guard;
use crate::{ChangeSet, Domain, Snapshot};

/// One page's check of a queue restored from disk: how many changes it
/// dropped, or `None` while the snapshot lacks the part it checks
/// against.
pub type Recheck = fn(&mut ChangeSet, &Snapshot) -> Option<usize>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A guard has not checked a restored change of its domains yet.
    GuardsWaiting { guard: Guard },
}

/// The guards a queue restored from disk has not met yet. It was queued
/// again before the system was known, so each guard runs once, with the
/// first snapshot that holds what it checks against.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Unchecked {
    waiting: BTreeSet<Guard>,
}

impl Unchecked {
    pub fn all() -> Self {
        Self {
            waiting: Guard::iter().collect(),
        }
    }

    /// The guards that have not run yet, in the order they run.
    pub fn waiting(&self) -> impl Iterator<Item = Guard> + '_ {
        self.waiting.iter().copied()
    }

    /// Whether `guard` has not run yet.
    pub fn waits(&self, guard: Guard) -> bool {
        self.waiting.contains(&guard)
    }

    /// # Errors
    ///
    /// [`Refusal::GuardsWaiting`] while a guard that has not run yet may
    /// still drop a change `changes` holds. A plan made then could run
    /// what the page refuses.
    pub fn ready(&self, changes: &ChangeSet) -> Result<(), Refusal> {
        self.holding(changes)
            .next()
            .map_or(Ok(()), |guard| Err(Refusal::GuardsWaiting { guard }))
    }

    /// The guards that have not run yet and may still drop a change
    /// `changes` holds, in the order they run, so a caller reads every
    /// part they wait for at once.
    pub fn holding<'a>(&'a self, changes: &'a ChangeSet) -> impl Iterator<Item = Guard> + 'a {
        self.waiting()
            .filter(|guard| guard.domains().iter().any(|domain| changes.holds(domain)))
    }

    /// Runs the waiting guards `snapshot` has the parts for. Returns how
    /// many changes they dropped, each with a line in the log.
    pub fn recheck(&mut self, changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
        let mut dropped = 0;

        for guard in Guard::iter() {
            if !self.waits(guard) {
                continue;
            }

            if let Some(count) = guard.recheck()(changes, snapshot) {
                self.waiting.remove(&guard);
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
