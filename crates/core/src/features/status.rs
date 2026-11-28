//! Decisions about the system status shown on Home.

#[cfg(test)]
mod tests;

use jiff::{SignedDuration, Timestamp};

/// After this many days without an update the page warns.
pub const STALE_AFTER_DAYS: u64 = 14;

const SECONDS_PER_DAY: u64 = 86_400;

const GIGABYTE: u64 = 1_000_000_000;

/// Free space below this share of the disk is low, on a disk of 100 GB
/// or less.
const LOW_DISK_SHARE: u64 = 10;

/// Free space below this is low on any disk bigger than 100 GB, where a
/// tenth would warn with plenty of room left.
const LOW_DISK_BYTES: u64 = 10 * GIGABYTE;

/// A package cache above this is worth cleaning.
pub const BIG_PACKAGE_CACHE: u64 = 5 * GIGABYTE;

/// A step Home offers under its status, in the order it shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextStep {
    /// The changes waiting in the queue.
    Review {
        queued: usize,
    },
    Setup,
    Maintenance,
    Diagnostics,
}

/// The whole days from `last` to `now`. A `last` after `now` (a clock set
/// back) is 0 days ago.
pub fn days_since(last: Timestamp, now: Timestamp) -> u64 {
    u64::try_from(now.duration_since(last).as_secs()).unwrap_or(0) / SECONDS_PER_DAY
}

/// Whether `last_update` lies more than `days` days before `now`. A last
/// update after `now` (a clock set back) is not stale.
pub fn is_stale(last_update: Timestamp, now: Timestamp, days: u64) -> bool {
    let limit = SignedDuration::from_secs(
        i64::try_from(days.saturating_mul(SECONDS_PER_DAY)).unwrap_or(i64::MAX),
    );

    now.duration_since(last_update) > limit
}

/// Whether `free` of `total` bytes is low: below a tenth of the disk or
/// below 10 GB, whichever is less.
pub fn disk_low(free: u64, total: u64) -> bool {
    free < (total / LOW_DISK_SHARE).min(LOW_DISK_BYTES)
}

/// Whether Home offers to clean up: some orphan is installed or the
/// package cache is big.
pub const fn cleanup_due(orphans: usize, package_cache: u64) -> bool {
    orphans > 0 || package_cache > BIG_PACKAGE_CACHE
}

/// Review while changes wait, the Setup Assistant until it ran once on an
/// installed system, then Maintenance and Diagnostics.
pub fn next_steps(queued: usize, live: bool, setup_finished: bool) -> Vec<NextStep> {
    let mut steps = Vec::new();

    if queued > 0 {
        steps.push(NextStep::Review { queued });
    }

    if !live && !setup_finished {
        steps.push(NextStep::Setup);
    }

    steps.extend([NextStep::Maintenance, NextStep::Diagnostics]);
    steps
}

/// The clean-up row says only the cache when no orphan is installed.
pub const fn cleanup_detail_id(orphans: usize) -> &'static str {
    if orphans == 0 {
        "home-cleanup-detail-cache"
    } else {
        "home-cleanup-detail"
    }
}
