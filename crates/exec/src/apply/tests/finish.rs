use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{RootAction, StepId};

use super::{FakeRunner, finished, request, run};

/// The command line's first Ctrl+C: the running step ends on its own, the
/// next one is reported cancelled without running, so the plan does not
/// look finished.
#[test]
fn a_finish_lets_the_running_step_end_and_cancels_the_next_unrun() {
    let mut runner = FakeRunner {
        finish_after: Some(1),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(
            Mode::Run,
            vec![RootAction::RefreshKeyring, RootAction::CleanPackageCache],
        ),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran, ["pacman-key --refresh-keys"]);
    assert_eq!(
        finished(&events),
        [
            (StepId::new(1), Outcome::Succeeded),
            (StepId::new(2), Outcome::Cancelled),
        ]
    );
    assert!(events.contains(&Event::Started {
        step: StepId::new(2)
    }));
    assert_eq!(events.last(), Some(&Event::Done));
}

/// A finish asked for before anything ran, such as during the password
/// dialog, runs nothing.
#[test]
fn a_finish_before_the_first_step_runs_nothing() {
    let mut runner = FakeRunner {
        finish_after: Some(0),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![RootAction::RefreshKeyring]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran, Vec::<String>::new());
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Cancelled)]);
}
