use toolbox_plan::StepId;
use toolbox_plan::procedure::{Command, Invocation, Program};
use toolbox_plan::protocol::{Event, Failure, Outcome};

use super::super::run_procedure;
use super::{FakeRunner, finished, wifi};

/// A cleanup that panics must not keep the cleanups after it from
/// running: each one puts back a part of the system.
#[test]
fn a_panic_in_a_cleanup_fails_it_and_the_later_cleanups_still_run() {
    let mut runner = FakeRunner {
        panics_at: Some("start -- first"),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();
    let procedure = [
        Invocation::Run(Command::new(Program::Systemctl).args(["stop", "--", "first"])),
        Invocation::Always(Command::new(Program::Systemctl).args(["start", "--", "first"])),
        Invocation::Always(Command::new(Program::Systemctl).args(["start", "--", "second"])),
    ];

    let outcome = run_procedure(StepId::new(1), &procedure, &mut runner, &mut events);

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    assert_eq!(
        runner.ran,
        [
            "systemctl stop -- first",
            "always: systemctl start -- first",
            "always: systemctl start -- second"
        ]
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::Output { step: _, data } if data.contains("stopped unexpectedly")
        )),
        "{events:?}"
    );
}

/// A panic that unwound through `apply` would end the helper with
/// NetworkManager stopped and the app waiting for `done`.
#[test]
fn a_panic_inside_a_step_still_runs_its_always_part_and_reports_done() {
    let mut runner = FakeRunner {
        panics_at: Some("disable --now"),
        ..FakeRunner::default()
    };

    let (ok, events) = wifi(&mut runner);

    assert!(!ok);
    assert_eq!(
        runner.ran,
        [
            "systemctl stop -- NetworkManager.service",
            "systemctl disable --now -- wpa_supplicant.service",
            "always: systemctl start -- NetworkManager.service"
        ]
    );
    assert_eq!(
        finished(&events),
        [(
            StepId::new(1),
            Outcome::Failed {
                cause: Failure::Unknown
            }
        )]
    );
    assert_eq!(events.last(), Some(&Event::Done));
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::Output { step: _, data } if data.contains("stopped unexpectedly")
        )),
        "{events:?}"
    );
}

#[test]
fn a_panic_in_the_first_step_skips_the_later_steps() {
    let mut runner = FakeRunner {
        panics_at: Some("stop -- NetworkManager"),
        ..FakeRunner::default()
    };

    let (ok, events) = wifi(&mut runner);

    assert!(!ok);
    assert_eq!(finished(&events).len(), 1, "{events:?}");
    assert_eq!(
        runner.ran.last().map(String::as_str),
        Some("always: systemctl start -- NetworkManager.service")
    );
}
