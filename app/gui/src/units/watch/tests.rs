use std::sync::Arc;
use std::time::Duration;

use futures_lite::future::Boxed;
use tokio::runtime::Handle;
use toolbox_plan::UnitScope;
use toolbox_system::CallError;
use toolbox_system::testing::unit_signals::ScriptedSignals;

use super::{Backoff, HEALTHY, MAX_RETRY, live};
use crate::units::{self, UnitList, UnitLister};
use crate::waking::Wake;

/// Answers no list, as a desktop without systemd does.
struct Silent;

impl UnitLister for Silent {
    fn list(&self, _scope: UnitScope) -> Boxed<UnitList> {
        let answer: UnitList = Err(CallError::Unavailable);
        Box::pin(async move { answer })
    }
}

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

/// The stream of the signals port ends with the result the port gives, and
/// the watcher reads that result.
#[tokio::test]
async fn the_live_stream_ends_with_the_result_of_the_signals_port() {
    let handle = units::spawn(
        &Handle::current(),
        Wake::none(),
        Arc::new(Silent),
        Duration::ZERO,
    );

    let signals = ScriptedSignals {
        signals: 2,
        ended: Err(CallError::Unavailable),
    };

    let ended = live(UnitScope::System, handle.requests(), Arc::new(signals)).await;

    assert_eq!(ended, Err(CallError::Unavailable));
}

#[test]
fn a_stream_that_ran_long_starts_the_waits_and_the_report_over() {
    let mut backoff = Backoff::default();
    for _ in 0_u8..6 {
        backoff.after(Duration::ZERO);
    }

    assert_eq!(backoff.after(HEALTHY), (secs(1), false));
}
