use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{Wake, channel};

fn counting() -> (Wake, Arc<AtomicUsize>) {
    let woken = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&woken);
    let wake = Wake::new(move || {
        counter.fetch_add(1, Ordering::Relaxed);
    });

    (wake, woken)
}

#[test]
fn every_value_sent_wakes_once() {
    let (wake, woken) = counting();
    let (sender, receiver) = channel(wake);

    let copy = sender.clone();
    sender.send(1_u8);
    copy.send(2_u8);

    assert_eq!(receiver.try_iter().collect::<Vec<_>>(), [1, 2]);
    assert_eq!(woken.load(Ordering::Relaxed), 2);
}

/// The command line has no window to wake.
#[test]
fn a_wake_for_nobody_still_delivers() {
    let (sender, receiver) = channel(Wake::none());

    sender.send("report");
    Wake::none().wake();

    assert_eq!(receiver.try_recv().ok(), Some("report"));
}
