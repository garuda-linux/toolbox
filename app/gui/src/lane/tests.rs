use std::time::{Duration, Instant};

use super::Lane;
use crate::testing::PATIENCE;

fn collect(lane: &Lane<u32>, want: usize) -> Vec<u32> {
    let started = Instant::now();
    let mut seen = Vec::new();

    while seen.len() < want && started.elapsed() < PATIENCE {
        seen.extend(lane.landed());
        std::thread::sleep(Duration::from_millis(5));
    }

    seen
}

#[test]
fn an_older_answer_that_lands_last_is_dropped() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let lane: Lane<u32> = Lane::new(runtime.handle().clone(), crate::waking::Wake::none());
    let (release, released) = crossbeam_channel::bounded::<()>(1);

    lane.blocking(
        "status",
        move || {
            released.recv().ok();
            1
        },
        Result::unwrap_or_default,
    );
    lane.blocking("status", || 2, Result::unwrap_or_default);

    assert_eq!(collect(&lane, 1), vec![2]);

    release.send(()).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(lane.landed().is_empty(), "the older answer landed");
}

#[test]
fn answers_of_other_kinds_all_land() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let lane: Lane<u32> = Lane::new(runtime.handle().clone(), crate::waking::Wake::none());

    lane.blocking("status", || 1, Result::unwrap_or_default);
    lane.future("hardware", async { 2 });

    let mut seen = collect(&lane, 2);
    seen.sort_unstable();
    assert_eq!(seen, vec![1, 2]);
}

#[test]
fn a_panicking_job_lands_as_its_message() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let lane: Lane<u32> = Lane::new(runtime.handle().clone(), crate::waking::Wake::none());

    lane.blocking(
        "status",
        || -> u32 { panic!("probe died") },
        |answer| u32::from(answer.is_err()),
    );

    assert_eq!(collect(&lane, 1), vec![1]);
}
