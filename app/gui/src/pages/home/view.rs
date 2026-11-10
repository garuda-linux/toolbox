//! What Home's status shows: the counts of the checks that worked, the
//! checks that failed, the pending updates and the facts besides them.

#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::path::PathBuf;
use std::time::SystemTime;

use toolbox_core::features::status::{
    STALE_AFTER_DAYS, cleanup_due, days_since, disk_low, is_stale,
};
use toolbox_plan::UnitName;
use toolbox_system::{Facts, HealthLevel, PendingUpdate, ProbeError, Status};

/// One of the status checks, for the words of its failure and of the all
/// clear. The order is the order the all clear lists them in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Updates,
    Aur,
    Merges,
    LastUpdate,
    Disk,
    Services,
    Cleanup,
    Health,
}

/// The status as Home shows it. A check that did not work counts as
/// nothing found, `failed` lists it with its reason.
#[derive(Debug)]
pub struct Overview<'status> {
    pub updates: usize,
    pub aur_updates: usize,
    /// The repositories' updates, then the AUR's.
    pub pending: Vec<Pending<'status>>,
    pub merges: &'status [PathBuf],
    /// Days since the last update, `None` when the log names none.
    pub days: Option<u64>,
    pub stale: bool,
    /// The worst level garuda-health found, `None` below a warning.
    pub health: Option<HealthLevel>,
    pub health_fixable: bool,
    /// garuda-health's messages above the level of a note.
    pub health_messages: Vec<&'status str>,
    /// What the facts read adds, quiet until it was read.
    pub machine: MachineChecks<'status>,
    pub failed: Vec<(Check, Cow<'status, str>)>,
    /// The checks that gave an answer, in the order the all clear lists.
    pub ran: Vec<Check>,
}

/// The checks of the facts read: the disk, the services and the clean-up.
#[derive(Debug, Default)]
pub struct MachineChecks<'status> {
    pub disk_low: bool,
    pub failed_units: &'status [UnitName],
    pub orphans: usize,
    /// Bytes in pacman's package cache.
    pub package_cache: u64,
    /// Orphans or a big cache are worth cleaning up.
    pub cleanup: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pending<'status> {
    pub name: &'status str,
    pub from: &'status str,
    pub to: &'status str,
    pub aur: bool,
}

impl<'status> Overview<'status> {
    /// Whether `check` could not be read.
    #[cfg(test)]
    pub fn failed_check(&self, check: Check) -> bool {
        self.failed.iter().any(|(failed, _)| *failed == check)
    }

    pub fn of(status: &'status Status, now: SystemTime) -> Self {
        let updates = found(&status.updates);
        let aur_updates = found(&status.aur_updates);
        let issues = found(&status.health);
        let last = status.last_update.as_ref().ok().copied().flatten();

        Self {
            updates: updates.len(),
            aur_updates: aur_updates.len(),
            pending: listed(updates, false)
                .chain(listed(aur_updates, true))
                .collect(),
            merges: found(&status.merges),
            days: last.map(|last| days_since(last, now)),
            stale: last.is_some_and(|last| is_stale(last, now, STALE_AFTER_DAYS)),
            health: issues
                .iter()
                .map(|issue| issue.level)
                .max()
                .filter(|level| *level > HealthLevel::Info),
            health_fixable: issues.iter().any(|issue| issue.fix_available),
            health_messages: issues
                .iter()
                .filter(|issue| issue.level != HealthLevel::Info)
                .map(|issue| issue.message.as_str())
                .collect(),
            machine: MachineChecks::default(),
            ran: [
                (Check::Updates, status.updates.is_ok()),
                (Check::Aur, status.aur_updates.is_ok()),
                (Check::Merges, status.merges.is_ok()),
                (Check::Health, status.health.is_ok()),
            ]
            .into_iter()
            .filter_map(|(check, answered)| answered.then_some(check))
            .collect(),
            failed: [
                (Check::Updates, reason(&status.updates)),
                (Check::Aur, reason(&status.aur_updates)),
                (Check::Merges, reason(&status.merges)),
                (Check::LastUpdate, reason(&status.last_update)),
                (Check::Health, reason(&status.health)),
            ]
            .into_iter()
            .filter_map(|(check, reason)| Some((check, reason?)))
            .collect(),
        }
    }

    pub fn with_facts(mut self, facts: &'status Facts) -> Self {
        let orphans = facts.orphans.as_ref().map_or(0, Vec::len);
        let package_cache = facts.package_cache.as_ref().copied().unwrap_or_default();

        self.machine.disk_low = facts
            .disk
            .as_ref()
            .is_ok_and(|disk| disk_low(disk.free, disk.total));
        self.machine.failed_units = found(&facts.failed_units);
        self.machine.orphans = orphans;
        self.machine.package_cache = package_cache;
        self.machine.cleanup = cleanup_due(orphans, package_cache);

        let cleanup_ran = facts.orphans.is_ok() || facts.package_cache.is_ok();
        let added = [
            (Check::Disk, facts.disk.is_ok()),
            (Check::Services, facts.failed_units.is_ok()),
            (Check::Cleanup, cleanup_ran),
        ];

        // Health comes last in the list, after the facts.
        let health = self.ran.pop_if(|check| *check == Check::Health);
        self.ran.extend(
            added
                .into_iter()
                .filter_map(|(check, ran)| ran.then_some(check)),
        );
        self.ran.extend(health);

        let cleanup_failed = reason(&facts.orphans).or_else(|| reason(&facts.package_cache));
        self.failed.extend(
            [
                (Check::Disk, reason(&facts.disk)),
                (Check::Services, reason(&facts.failed_units)),
                (Check::Cleanup, cleanup_failed),
            ]
            .into_iter()
            .filter_map(|(check, reason)| Some((check, reason?))),
        );

        self
    }

    /// Whether Home has a row to show, so it can say that all is well.
    pub const fn attention(&self, reboot_pending: bool) -> bool {
        self.updates > 0
            || self.aur_updates > 0
            || !self.merges.is_empty()
            || self.stale
            || self.health.is_some()
            || self.machine.disk_low
            || !self.machine.failed_units.is_empty()
            || self.machine.cleanup
            || reboot_pending
    }
}

impl Check {
    /// Stable code, part of the text ids.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Updates => "updates",
            Self::Aur => "aur",
            Self::Merges => "merges",
            Self::LastUpdate => "last-update",
            Self::Disk => "disk",
            Self::Services => "services",
            Self::Cleanup => "cleanup",
            Self::Health => "health",
        }
    }
}

fn found<T>(result: &Result<Vec<T>, ProbeError>) -> &[T] {
    result.as_deref().unwrap_or_default()
}

fn listed(updates: &[PendingUpdate], aur: bool) -> impl Iterator<Item = Pending<'_>> {
    updates.iter().map(move |update| Pending {
        name: update.name.as_str(),
        from: &update.from,
        to: &update.to,
        aur,
    })
}

fn reason<T>(result: &Result<T, ProbeError>) -> Option<Cow<'_, str>> {
    match result {
        Err(ProbeError::Failed(reason)) => Some(Cow::Borrowed(reason)),
        Err(error @ ProbeError::Io { what: _, error: _ }) => error.reason().map(Cow::Owned),
        Ok(_) | Err(ProbeError::Unavailable) => None,
    }
}
