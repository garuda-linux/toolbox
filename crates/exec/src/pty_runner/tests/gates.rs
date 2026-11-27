use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use toolbox_plan::StepId;
use toolbox_plan::protocol::{Answer, Event};
use toolbox_pty::{PtyCommand, Session};

use super::{Environment, PtyRunner};
use crate::apply::{EventSink, Runner as _};

/// Long enough for a wait to report itself on a loaded test machine.
const SOON: Duration = Duration::from_secs(10);

/// Hands the events of a wait to the test thread.
struct Heard(Sender<Event>);

impl EventSink for Heard {
    fn send(&mut self, event: Event) {
        self.0.send(event).ok();
    }
}

/// Runs `work` on a named thread of its own, as the helper names its threads.
fn spawn<F, T>(work: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new()
        .name("test-gate".to_owned())
        .spawn(work)
        .unwrap()
}

/// Waits at the gate of step 2 on another thread. The events come back on
/// the second channel, so the test answers only once the wait reported.
fn waiting_at_gate(answers: Receiver<Answer>) -> (JoinHandle<bool>, Receiver<Event>) {
    let (heard, events) = channel();

    let handle = spawn(move || {
        let mut runner = PtyRunner::new(answers, Environment::Clean);
        runner.wait_to_proceed(StepId::new(2), &mut Heard(heard))
    });

    (handle, events)
}

fn waiting(step: u32) -> Event {
    Event::Waiting {
        step: StepId::new(step),
    }
}

fn proceed(step: u32) -> Answer {
    Answer::Proceed {
        step: StepId::new(step),
    }
}

/// A wait at the gate of step 2, once the wait has reported it.
fn parked_at_gate() -> (Sender<Answer>, JoinHandle<bool>) {
    let (app, answers) = channel();
    let (handle, events) = waiting_at_gate(answers);

    assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
    (app, handle)
}

/// A proceed for step 5 reaches the wait of step 2. The cancel after it
/// ends the wait only if the proceed did not pass the gate.
#[test]
fn a_proceed_for_another_step_does_not_pass_the_gate() {
    let (app, handle) = parked_at_gate();

    app.send(proceed(5)).unwrap();
    app.send(Answer::Cancel).unwrap();

    assert!(!handle.join().unwrap());
}

/// Queues `answers` before the wait, then closes the app, and waits at the
/// gate of `step`. Returns whether the gate was passed and what was reported.
fn queued_wait(answers: Vec<Answer>, step: u32) -> (bool, Vec<Event>) {
    let (app, receiver) = channel();
    for answer in answers {
        app.send(answer).unwrap();
    }

    drop(app);

    let mut runner = PtyRunner::new(receiver, Environment::Clean);
    let mut events = Vec::new();
    let passed = runner.wait_to_proceed(StepId::new(step), &mut events);

    (passed, events)
}

/// A proceed for step 2 sent before its own gate is dropped with the other
/// stale answers, so it does not pass the gate.
#[test]
fn a_proceed_sent_before_the_gate_does_not_pass_it() {
    let (passed, events) = queued_wait(vec![proceed(2)], 2);

    assert!(!passed);
    assert_eq!(events, [waiting(2)]);
}

/// The proceed for step 2 was sent for an earlier gate. The gate of step 3
/// must not take it.
#[test]
fn a_proceed_for_an_earlier_gate_does_not_pass_a_later_one() {
    let (passed, events) = queued_wait(vec![proceed(2)], 3);

    assert!(!passed);
    assert_eq!(events, [waiting(3)]);
}

#[test]
fn a_proceed_at_the_gate_passes_it() {
    let (app, answers) = channel();
    let (handle, events) = waiting_at_gate(answers);

    assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
    app.send(Answer::Proceed {
        step: StepId::new(2),
    })
    .unwrap();

    assert!(handle.join().unwrap());
}

#[test]
fn a_cancel_or_a_finish_at_the_gate_ends_the_wait() {
    for answer in [Answer::Cancel, Answer::Finish] {
        let (app, answers) = channel();
        let (handle, events) = waiting_at_gate(answers);

        assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
        app.send(answer).unwrap();

        assert!(!handle.join().unwrap());
    }
}

#[test]
fn a_gone_app_ends_the_wait() {
    let (app, answers) = channel();
    let (handle, events) = waiting_at_gate(answers);

    assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
    drop(app);

    assert!(!handle.join().unwrap());
}

/// What SIGTERM sets in the helper. The app is still there, so only the
/// flag can end the wait.
#[test]
fn the_stop_flag_ends_the_wait() {
    let (_app, answers) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    let (heard, events) = channel();
    let flag = Arc::clone(&stop);

    let handle = spawn(move || {
        let mut runner = PtyRunner::new(answers, Environment::Clean).stop_on(flag);
        runner.wait_to_proceed(StepId::new(2), &mut Heard(heard))
    });

    assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
    stop.store(true, Ordering::Relaxed);

    assert!(!handle.join().unwrap());
}

/// Keystrokes and secrets have no program to reach at a gate. They must
/// neither end the wait nor wait for a later step.
#[test]
fn input_at_the_gate_neither_passes_nor_stops_it() {
    let (app, answers) = channel();
    let (handle, events) = waiting_at_gate(answers);

    assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
    app.send(Answer::Input {
        data: "yes\n".to_owned(),
    })
    .unwrap();
    app.send(Answer::Proceed {
        step: StepId::new(2),
    })
    .unwrap();

    assert!(handle.join().unwrap());
}

/// A proceed that arrives while a program runs is no answer to a gate that
/// does not exist yet, so it must not pass the next one.
#[test]
fn a_proceed_during_a_step_is_not_kept_for_a_later_gate() {
    let (app, answers) = channel();
    app.send(Answer::Proceed {
        step: StepId::new(2),
    })
    .unwrap();
    let mut runner = PtyRunner::new(answers, Environment::Clean);
    let mut session =
        Session::spawn(&PtyCommand::new("/bin/sh").args(["-c", "sleep 0.3"])).unwrap();

    runner
        .pump(StepId::new(1), &mut session, &mut Vec::new())
        .unwrap();

    let (heard, events) = channel();
    let handle = spawn(move || runner.wait_to_proceed(StepId::new(2), &mut Heard(heard)));

    assert_eq!(events.recv_timeout(SOON), Ok(waiting(2)));
    thread::sleep(Duration::from_millis(300));
    assert!(!handle.is_finished());

    app.send(Answer::Cancel).unwrap();
    assert!(!handle.join().unwrap());
}
