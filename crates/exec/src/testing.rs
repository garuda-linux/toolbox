//! Helpers the tests of several modules share.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::time::{Duration, Instant};

use toolbox_plan::StepId;
use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::{Event, Failure, Outcome};

use crate::apply::EventSink;

impl EventSink for Vec<Event> {
    fn send(&mut self, event: Event) {
        self.push(event);
    }
}

pub fn printed(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Output { step: _, data } => Some(data.as_str()),
            Event::Accepted
            | Event::Rejected { reason: _ }
            | Event::Started { step: _ }
            | Event::Finished {
                step: _,
                outcome: _,
            }
            | Event::Done => None,
        })
        .collect()
}

/// Runs `builtin` below `root` without a home, as the helper does, and
/// returns how it ended and what it printed.
pub fn run_builtin(root: &Path, builtin: &Builtin) -> (Outcome, String) {
    let mut events = Vec::new();
    let outcome = crate::builtin::execute(root, None, StepId::new(1), builtin, &mut events);

    (outcome, printed(&events))
}

pub fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

/// Fails the test unless `outcome` is a failure without a known cause.
#[track_caller]
pub fn assert_failed(outcome: Outcome) {
    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown
        },
        "the step did not fail"
    );
}

/// Whether the process whose id a script wrote to `pid_file` has ended,
/// waiting a moment for the kernel to end it after a kill.
pub fn gone(pid_file: &Path) -> bool {
    // Far below the sleeps of the scripts, far above a loaded test machine.
    const SOON: Duration = Duration::from_secs(10);

    let pid = fs::read_to_string(pid_file).unwrap();
    let process = Path::new("/proc").join(pid.trim());
    let started = Instant::now();

    while process.exists() && started.elapsed() < SOON {
        std::thread::sleep(Duration::from_millis(10));
    }

    !process.exists()
}
