use std::time::{Duration, Instant};

use super::{Asked, Lane};
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
    lane.future("hardware", async { 2 }, Result::unwrap_or_default);

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

fn died() -> u32 {
    panic!("read died")
}

#[test]
fn a_panicking_future_lands_as_an_error_answer() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let lane: Lane<u32> = Lane::new(runtime.handle().clone(), crate::waking::Wake::none());

    lane.future("connectivity", async { died() }, |answer| {
        u32::from(answer.is_err_and(|error| error.is_panic()))
    });

    assert_eq!(collect(&lane, 1), vec![1]);
}

/// An `Asked` whose reads count up from 1 and wait for `released`.
fn counting(runtime: &tokio::runtime::Runtime) -> (Asked<u32>, crossbeam_channel::Sender<()>) {
    let (release, released) = crossbeam_channel::unbounded::<()>();
    let runs = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let asked = Asked::new(
        runtime.handle().clone(),
        crate::waking::Wake::none(),
        move |lane: &Lane<u32>| {
            let runs = std::sync::Arc::clone(&runs);
            let released = released.clone();
            lane.blocking(
                "count",
                move || {
                    released.recv().ok();
                    runs.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
                },
                Result::unwrap_or_default,
            );
        },
    );

    (asked, release)
}

fn taken(asked: &mut Asked<u32>) -> Option<u32> {
    let started = Instant::now();
    while started.elapsed() < PATIENCE {
        if let Some(answer) = asked.take() {
            return Some(answer);
        }

        std::thread::sleep(Duration::from_millis(5));
    }

    None
}

#[test]
fn an_asked_read_lands_once() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (mut asked, release) = counting(&runtime);

    assert!(asked.read(), "the first ask starts a read");
    release.send(()).unwrap();

    assert_eq!(taken(&mut asked), Some(1));
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(asked.take(), None);
}

#[test]
fn asks_while_a_read_runs_become_one_more_read() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (mut asked, release) = counting(&runtime);

    assert!(asked.read());
    assert!(!asked.read(), "a second read started beside the first");
    assert!(!asked.read());
    release.send(()).unwrap();
    assert_eq!(taken(&mut asked), Some(1));

    release.send(()).unwrap();
    assert_eq!(taken(&mut asked), Some(2));

    release.send(()).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(asked.take(), None, "the asks made more than one more read");
}
