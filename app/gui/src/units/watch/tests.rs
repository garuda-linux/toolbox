use std::time::Duration;

use super::{Backoff, HEALTHY, MAX_RETRY, RELISTING};

const fn secs(count: u64) -> Duration {
    Duration::from_secs(count)
}

#[test]
fn waits_double_up_to_the_limit() {
    let mut backoff = Backoff::default();

    let waits: Vec<Duration> = std::iter::repeat_with(|| backoff.after(Duration::ZERO).0)
        .take(7)
        .collect();

    assert_eq!(
        waits,
        [
            secs(1),
            secs(2),
            secs(4),
            secs(8),
            secs(16),
            MAX_RETRY,
            MAX_RETRY
        ]
    );
}

/// No bus at all would otherwise warn every 30 s for as long as the app
/// runs.
#[test]
fn only_the_first_failure_of_an_outage_is_reported() {
    let mut backoff = Backoff::default();

    assert!(!backoff.after(Duration::ZERO).1);
    assert!(backoff.after(Duration::ZERO).1);
}

/// A job's start comes with its end, and property changes come from
/// every running timer, so neither lists again.
#[test]
fn only_changes_of_the_units_and_their_files_list_again() {
    for quiet in ["PropertiesChanged", "JobNew", "StartupFinished"] {
        assert!(!RELISTING.contains(&quiet), "{quiet}");
    }

    assert!(RELISTING.contains(&"JobRemoved"));
    assert!(RELISTING.contains(&"UnitFilesChanged"));
}

#[test]
fn a_stream_that_ran_long_starts_the_waits_and_the_report_over() {
    let mut backoff = Backoff::default();
    for _ in 0_u8..6 {
        backoff.after(Duration::ZERO);
    }

    assert_eq!(backoff.after(HEALTHY), (secs(1), false));
}
