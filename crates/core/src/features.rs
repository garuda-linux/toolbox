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

use crate::domains::{
    AccountLocks, AurPackages, FullNames, Groups, GrubDefaults, Hblock, HomeFolders, Locales,
    LoginShells, ModuleBlacklist, NewGroups, Packages, PlymouthThemes, RemovedGroups, RemovedUsers,
    SdBootKernelDomain, WifiBackends,
};
use crate::{ChangeSet, Domain, Snapshot};

/// Every guard by its id, in the order they run. A new page that queues
/// changes adds its line here.
pub const GUARDS: &[Guard] = &[
    Guard {
        id: "users",
        domains: &[
            RemovedUsers::ID,
            Groups::ID,
            LoginShells::ID,
            FullNames::ID,
            HomeFolders::ID,
        ],
        run: users::recheck,
    },
    Guard {
        id: "memberships",
        domains: &[Groups::ID, RemovedGroups::ID],
        run: users::recheck_groups,
    },
    // Apart from the users: the lock flags come from the helper once the
    // Users page asks, long after the account files.
    Guard {
        id: USER_LOCKS,
        domains: &[RemovedUsers::ID, Groups::ID, AccountLocks::ID],
        run: users::recheck_locks,
    },
    Guard {
        id: "kernels",
        domains: &[Packages::ID],
        run: kernels::recheck,
    },
    Guard {
        id: "locales",
        domains: &[Locales::ID],
        run: locales::recheck,
    },
    Guard {
        id: "language-packs",
        domains: &[Packages::ID],
        run: locales::view::drop_stranded_packs,
    },
    Guard {
        id: "plymouth",
        domains: &[PlymouthThemes::ID, SdBootKernelDomain::ID],
        run: boot::recheck,
    },
    Guard {
        id: "grub",
        domains: &[GrubDefaults::ID, SdBootKernelDomain::ID],
        run: boot::recheck_grub,
    },
    Guard {
        id: "module-blacklist",
        domains: &[ModuleBlacklist::ID],
        run: network::recheck_blacklist,
    },
    Guard {
        id: "new-groups",
        domains: &[NewGroups::ID],
        run: missing::recheck_new_groups,
    },
    Guard {
        id: "hblock",
        domains: &[Hblock::ID],
        run: missing::recheck_hblock,
    },
    Guard {
        id: "wifi-backend",
        domains: &[WifiBackends::ID],
        run: missing::recheck_wifi_backend,
    },
    // Waits for the AUR's copy, which the window reads only on demand.
    Guard {
        id: AUR_PACKAGES,
        domains: &[AurPackages::ID],
        run: packages::aur::recheck,
    },
];

/// The guard that waits for the lock flags, which only a read through
/// the helper gives.
pub const USER_LOCKS: &str = "user-locks";

/// The guard that waits for the names of the AUR's copy.
pub const AUR_PACKAGES: &str = "aur-packages";

/// One page's check of a queue restored from disk: how many changes it
/// dropped, or `None` while the snapshot lacks the part it checks
/// against.
pub type Recheck = fn(&mut ChangeSet, &Snapshot) -> Option<usize>;

#[derive(Clone, Copy, Debug)]
pub struct Guard {
    pub id: &'static str,
    /// The domains whose changes it may drop, so a plan of them waits
    /// for it.
    pub domains: &'static [&'static str],
    pub run: Recheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A guard has not checked a restored change of its domains yet.
    GuardsWaiting { guard: &'static str },
}

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
            waiting: GUARDS.iter().map(|guard| guard.id).collect(),
        }
    }

    /// The ids of the guards that have not run yet.
    pub fn waiting(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.waiting.iter().copied()
    }

    /// Whether `guard` has not run yet.
    pub fn waits(&self, guard: &str) -> bool {
        self.waiting.contains(guard)
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
    pub fn holding<'a>(
        &'a self,
        changes: &'a ChangeSet,
    ) -> impl Iterator<Item = &'static str> + 'a {
        GUARDS
            .iter()
            .filter(|guard| {
                self.waits(guard.id) && guard.domains.iter().any(|domain| changes.holds(domain))
            })
            .map(|guard| guard.id)
    }

    /// Runs the waiting guards `snapshot` has the parts for. Returns how
    /// many changes they dropped, each with a line in the log.
    pub fn recheck(&mut self, changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
        let mut dropped = 0;

        for guard in GUARDS {
            if !self.waiting.contains(guard.id) {
                continue;
            }

            if let Some(count) = (guard.run)(changes, snapshot) {
                self.waiting.remove(guard.id);
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
