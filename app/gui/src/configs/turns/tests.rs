use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use parking_lot::Mutex;

use super::Turns;
use crate::testing::until;

#[test]
fn a_later_turn_waits_until_the_earlier_one_ended() {
    let turns = Turns::default();
    let first = turns.take();
    let mut second = turns.take();
    let order = Arc::new(Mutex::new(Vec::new()));
    let started = Arc::new(AtomicBool::new(false));

    let later = {
        let order = Arc::clone(&order);
        let started = Arc::clone(&started);
        thread::Builder::new()
            .name("turns-test".to_owned())
            .spawn(move || {
                started.store(true, Ordering::SeqCst);
                second.wait();
                order.lock().push("second");
            })
            .unwrap()
    };

    until("the later turn never started", || {
        started.load(Ordering::SeqCst)
    });
    order.lock().push("first");
    drop(first);
    later.join().unwrap();

    assert_eq!(*order.lock(), ["first", "second"]);
}

#[test]
fn a_turn_that_ends_without_its_work_still_lets_the_next_one_go() {
    let turns = Turns::default();
    let first = turns.take();
    let mut second = turns.take();

    drop(first);
    second.wait();
}

#[test]
fn the_first_turn_waits_for_nothing() {
    let turns = Turns::default();
    turns.take().wait();
}
