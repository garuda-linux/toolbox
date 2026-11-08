use std::time::{Duration, SystemTime};

use super::{SECONDS_PER_DAY as DAY, cleanup_due, days_since, disk_low, is_stale};

const GB: u64 = 1_000_000_000;

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

#[test]
fn days_since_counts_whole_days_and_none_for_a_clock_set_back() {
    let last = at(100 * DAY);

    assert_eq!(days_since(last, at(100 * DAY + DAY - 1)), 0);
    assert_eq!(days_since(last, at(103 * DAY)), 3);
    assert_eq!(days_since(last, at(DAY)), 0);
}

#[test]
fn stale_only_after_the_given_days() {
    let last = at(100 * DAY);

    assert!(!is_stale(last, at(100 * DAY), 14));
    assert!(!is_stale(last, at(114 * DAY), 14));
    assert!(is_stale(last, at(114 * DAY + 1), 14));
    assert!(is_stale(last, at(200 * DAY), 14));
}

/// A clock set back makes the last update look like the future.
#[test]
fn an_update_in_the_future_is_not_stale() {
    assert!(!is_stale(at(10 * DAY), at(DAY), 14));
}

#[test]
fn zero_days_is_stale_after_any_time() {
    assert!(!is_stale(at(DAY), at(DAY), 0));
    assert!(is_stale(at(DAY), at(DAY + 1), 0));
}

#[test]
fn huge_day_counts_never_overflow() {
    assert!(!is_stale(at(0), at(400 * DAY), u64::MAX));
}

#[test]
fn a_big_disk_is_low_below_ten_gigabytes() {
    assert!(disk_low(9 * GB, 500 * GB));
    assert!(!disk_low(11 * GB, 500 * GB));
}

#[test]
fn a_small_disk_is_low_below_a_tenth() {
    assert!(disk_low(6 * GB, 64 * GB));
    assert!(!disk_low(7 * GB, 64 * GB));
}

#[test]
fn an_empty_answer_is_no_low_disk() {
    assert!(!disk_low(0, 0));
}

#[test]
fn a_cleanup_is_due_for_any_orphan_or_a_big_cache() {
    assert!(cleanup_due(1, 0));
    assert!(cleanup_due(0, 5 * GB + 1));
    assert!(!cleanup_due(0, 5 * GB));
}
