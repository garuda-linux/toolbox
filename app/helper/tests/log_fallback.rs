//! The helper installs no tracing subscriber: what its shared crates
//! trace must still reach its `log` logger (tracing's `log` feature).
//! A process of its own, since the `log` logger is set once.

use std::sync::OnceLock;
use std::sync::mpsc::{self, Sender};

use toolbox_plan::protocol::{Event, Mode};
use toolbox_plan::{RootAction, Step, StepId};

static RECORDS: OnceLock<Sender<String>> = OnceLock::new();
static LOGGER: Capture = Capture;

struct Capture;

impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if let Some(records) = RECORDS.get() {
            records
                .send(format!("{} {}", record.target(), record.args()))
                .ok();
        }
    }

    fn flush(&self) {}
}

struct Events(Vec<Event>);

impl toolbox_exec::EventSink for Events {
    fn send(&mut self, event: Event) {
        self.0.push(event);
    }
}

#[test]
fn tracing_in_toolbox_exec_reaches_the_log_logger() {
    let (sender, records) = mpsc::channel();
    RECORDS.set(sender).unwrap();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let steps = [Step {
        id: StepId::new(1),
        action: RootAction::RefreshKeyring,
    }];

    let (answers, received) = mpsc::channel();
    drop(answers);
    let mut runner = toolbox_exec::PtyRunner::new(received, toolbox_exec::Environment::Clean);

    toolbox_exec::apply(Mode::DryRun, &steps, &mut runner, &mut Events(Vec::new()));

    let records: Vec<String> = records.try_iter().collect();
    assert!(
        records
            .iter()
            .any(|record| record.starts_with("toolbox_exec::apply ")
                && record.contains("step")
                && record.contains("index=1")
                && record.contains("pacman-key")),
        "{records:?}"
    );
}
