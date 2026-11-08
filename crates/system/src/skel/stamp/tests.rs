use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::backup_stamp;

fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[test]
fn the_epoch() {
    assert_eq!(backup_stamp(at(0)), "19700101-000000");
}

#[test]
fn a_leap_day() {
    assert_eq!(backup_stamp(at(951_782_400)), "20000229-000000");
    assert_eq!(backup_stamp(at(1_709_251_199)), "20240229-235959");
}

#[test]
fn the_end_of_a_year() {
    assert_eq!(backup_stamp(at(946_684_799)), "19991231-235959");
}

#[test]
fn a_century_without_leap_day() {
    assert_eq!(backup_stamp(at(4_107_542_400)), "21000301-000000");
}

#[test]
fn an_ordinary_moment() {
    assert_eq!(backup_stamp(at(1_791_551_107)), "20261009-130507");
}

#[test]
fn ignores_fractions_of_a_second() {
    assert_eq!(
        backup_stamp(at(1_791_551_107) + Duration::from_millis(999)),
        "20261009-130507"
    );
}

#[test]
fn a_time_before_the_epoch_counts_as_the_epoch() {
    assert_eq!(
        backup_stamp(UNIX_EPOCH - Duration::from_secs(5)),
        "19700101-000000"
    );
}
