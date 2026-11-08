use std::time::{Duration, Instant};

use super::{Cancellation, Signal};

const GRACE: Duration = Duration::from_secs(5);

fn after(start: Instant, seconds: u64) -> Instant {
    start + Duration::from_secs(seconds)
}

#[test]
fn nothing_is_sent_before_a_cancel() {
    let start = Instant::now();
    let mut cancellation = Cancellation::new(GRACE);

    assert!(!cancellation.is_requested());
    assert_eq!(cancellation.poll(after(start, 60)), None);
}

#[test]
fn interrupt_then_terminate_then_kill_each_after_the_grace() {
    let start = Instant::now();
    let mut cancellation = Cancellation::new(GRACE);

    assert_eq!(cancellation.start(start), Some(Signal::Interrupt));
    assert!(cancellation.is_requested());
    assert_eq!(cancellation.poll(after(start, 4)), None);
    assert_eq!(cancellation.poll(after(start, 5)), Some(Signal::Terminate));
    assert_eq!(cancellation.poll(after(start, 9)), None);
    assert_eq!(cancellation.poll(after(start, 10)), Some(Signal::Kill));
    assert_eq!(
        cancellation.poll(after(start, 60)),
        None,
        "kill is sent once"
    );
}

#[test]
fn a_second_cancel_does_not_restart_the_sequence() {
    let start = Instant::now();
    let mut cancellation = Cancellation::new(GRACE);

    assert_eq!(cancellation.start(start), Some(Signal::Interrupt));
    assert_eq!(cancellation.start(after(start, 3)), None);
    assert_eq!(cancellation.poll(after(start, 5)), Some(Signal::Terminate));
}

#[test]
fn signals_map_to_their_numbers() {
    assert_eq!(
        [Signal::Interrupt, Signal::Terminate, Signal::Kill]
            .map(|signal| Signal::to_rustix(signal).as_raw()),
        [2_i32, 15_i32, 9_i32]
    );
}
