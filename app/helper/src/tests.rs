use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use rustix::process::{Signal, getpid, kill_process};

use super::stop_on_signals;

/// Signals reach every handler of the process, and `cargo test` runs the
/// tests of one binary as threads of one process. A test that sends one
/// holds this, so another test's flag never sees it.
static SIGNALS: Mutex<()> = Mutex::new(());

/// Sent to this very process: without the handlers the first one would
/// end the test run.
#[test]
fn the_signals_that_end_a_program_only_set_the_flag() {
    let _signals = SIGNALS.lock();
    let stop = stop_on_signals();

    assert!(!stop.load(Ordering::Relaxed));

    for signal in [Signal::TERM, Signal::HUP] {
        stop.store(false, Ordering::Relaxed);
        kill_process(getpid(), signal).unwrap();

        let sent = Instant::now();

        while !stop.load(Ordering::Relaxed) && sent.elapsed() < Duration::from_secs(10) {
            thread::sleep(Duration::from_millis(5));
        }

        assert!(stop.load(Ordering::Relaxed), "{signal:?} set nothing");
    }
}

/// A terminal's Ctrl+C reaches the helper too. The command line answers
/// it with a finish or a cancel, so the helper itself neither ends nor
/// cancels.
#[test]
fn a_ctrl_c_neither_ends_nor_cancels_the_helper() {
    let _signals = SIGNALS.lock();
    let stop = stop_on_signals();

    kill_process(getpid(), Signal::INT).unwrap();
    thread::sleep(Duration::from_millis(100));

    assert!(!stop.load(Ordering::Relaxed));
}
