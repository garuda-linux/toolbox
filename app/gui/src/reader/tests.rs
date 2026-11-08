use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use toolbox_gui_kit::Loadable;

use super::{ReaderHandle, Watched, spawn, spawn_fallible};
use crate::testing::PATIENCE;

async fn reading_where(
    handle: &mut ReaderHandle<usize>,
    wanted: impl FnMut(&Loadable<usize>) -> bool,
) -> Loadable<usize> {
    tokio::time::timeout(PATIENCE, handle.reading.wait_for(wanted))
        .await
        .unwrap()
        .unwrap()
        .clone()
}

fn start<R>(read: R) -> ReaderHandle<usize>
where
    R: Fn() -> usize + Send + Sync + 'static,
{
    spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        "a number",
        read,
    )
}

/// A read that waits for the test before it answers with its run's
/// number, and counts its runs.
fn held_read() -> (
    impl Fn() -> usize + Send + Sync + 'static,
    Sender<()>,
    Arc<AtomicUsize>,
) {
    let (release, released): (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();
    let runs = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&runs);

    let read = move || {
        let run = counted.fetch_add(1, Ordering::SeqCst) + 1;
        released.recv_timeout(PATIENCE).unwrap();
        run
    };

    (read, release, runs)
}

#[tokio::test]
async fn the_first_read_starts_at_once() {
    let mut handle = start(|| 7);

    let reading = reading_where(&mut handle, |reading| reading.get().is_some()).await;

    assert!(!reading.is_loading());
    assert_eq!(reading.get(), Some(&7));
}

#[tokio::test]
async fn a_running_read_is_busy() {
    let (read, release, _runs) = held_read();
    let mut handle = start(read);

    reading_where(&mut handle, Loadable::is_loading).await;
    release.send(()).unwrap();

    reading_where(&mut handle, |reading| {
        !reading.is_loading() && reading.get().is_some()
    })
    .await;
}

#[tokio::test]
async fn requests_during_a_read_become_one_more_read() {
    let (read, release, runs) = held_read();
    let mut handle = start(read);

    reading_where(&mut handle, Loadable::is_loading).await;
    handle.refresh();
    handle.refresh();
    handle.refresh();
    release.send(()).unwrap();
    release.send(()).unwrap();

    let reading = reading_where(&mut handle, |reading| {
        reading.get() == Some(&2) && !reading.is_loading()
    })
    .await;

    assert_eq!(reading.get(), Some(&2));

    // A third read would have started by now.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(runs.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn a_new_reading_is_seen_once() {
    let mut handle = start(|| 1);

    let read = tokio::time::timeout(PATIENCE, async {
        loop {
            if let Some(reading) = handle.changed()
                && reading.get().is_some()
            {
                return reading;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();

    assert!(!read.is_loading());
    assert!(handle.changed().is_none());
}

#[tokio::test]
async fn a_period_reads_again_until_it_is_taken_back() {
    let runs = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&runs);
    let handle = start(move || counted.fetch_add(1, Ordering::SeqCst) + 1);

    handle.every(Some(Duration::from_millis(20)));
    tokio::time::timeout(PATIENCE, async {
        while runs.load(Ordering::SeqCst) < 3 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();

    handle.every(None);
    tokio::time::sleep(Duration::from_millis(50)).await;
    let stopped = runs.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(runs.load(Ordering::SeqCst), stopped);
}

#[tokio::test]
async fn a_watch_keeps_the_newest_reading() {
    let mut watched = Watched::new(Some(start(|| 7)));

    tokio::time::timeout(PATIENCE, async {
        while watched.latest().is_none() {
            tokio::time::sleep(Duration::from_millis(5)).await;
            watched.take();
        }
    })
    .await
    .unwrap();

    assert_eq!(watched.latest(), Some(&7));
    assert!(Watched::<usize>::new(None).latest().is_none());
}

#[tokio::test]
async fn a_failed_read_keeps_the_last_value_beside_its_error() {
    let runs = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&runs);
    let mut handle = spawn_fallible(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        "a number",
        move || match counted.fetch_add(1, Ordering::SeqCst) {
            0 => Ok(7),
            _ => Err("gone".to_owned()),
        },
    );

    reading_where(&mut handle, |reading| reading.get().is_some()).await;
    handle.refresh();
    let reading = reading_where(&mut handle, |reading| reading.error().is_some()).await;

    assert_eq!(reading.error().map(String::as_str), Some("gone"));
    assert_eq!(reading.get(), Some(&7));
}
