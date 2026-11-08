//! Decisions about the system status shown on Home.

#[cfg(test)]
mod tests;

use std::time::{Duration, SystemTime};

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

/// The whole days from `last` to `now`. A `last` after `now` (a clock set
/// back) is 0 days ago.
pub fn days_since(last: SystemTime, now: SystemTime) -> u64 {
    now.duration_since(last)
        .map_or(0, |since| since.as_secs() / SECONDS_PER_DAY)
}

/// Whether `last_update` lies more than `days` days before `now`. A last
/// update after `now` (a clock set back) is not stale.
pub fn is_stale(last_update: SystemTime, now: SystemTime, days: u64) -> bool {
    let limit = Duration::from_secs(days.saturating_mul(SECONDS_PER_DAY));

    now.duration_since(last_update).is_ok_and(|age| age > limit)
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
