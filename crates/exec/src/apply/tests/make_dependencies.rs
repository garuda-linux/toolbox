//! Only the names the orphan query printed go to `pacman -Rn`, once.

use std::collections::VecDeque;

use toolbox_plan::protocol::{Event, Failure, Mode, Outcome};
use toolbox_plan::{PackageName, RootAction, StepId, TargetList};

use super::dependencies::{answering, said};
use super::{FakeRunner, finished, request, run};

const QUERY: &str = "orphans: pacman -Qdtq -- cowsay tree";
const QUERY_OF_THREE: &str = "orphans: pacman -Qdtq -- bash cowsay tree";

fn removal<const N: usize>(names: [&str; N]) -> RootAction {
    RootAction::RemoveMakeDependencies {
        packages: TargetList::new(names.map(|name| PackageName::new(name).unwrap())).unwrap(),
    }
}

/// Runs the removal of `names` and returns whether the step held and what
/// the Console printed.
fn remove<const N: usize>(runner: &mut FakeRunner, names: [&str; N]) -> (bool, Vec<Event>) {
    let mut events = Vec::new();
    let ok = run(
        &request(Mode::Run, vec![removal(names)]),
        runner,
        &mut events,
    );

    (ok, events)
}

#[test]
fn the_names_the_query_printed_are_removed_with_one_pacman_call() {
    let mut runner = answering(&["cowsay"]);

    let (ok, events) = remove(&mut runner, ["cowsay", "tree"]);

    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Succeeded)]);
    assert_eq!(runner.ran, [QUERY, "pacman -Rn --noconfirm -- cowsay"]);
    assert!(ok);
}

#[test]
fn a_query_that_prints_nothing_removes_nothing_and_succeeds() {
    let mut runner = answering(&[]);

    let (ok, events) = remove(&mut runner, ["cowsay", "tree"]);

    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Skipped)]);
    assert_eq!(runner.ran, [QUERY]);
    assert!(ok);
}

/// Each answer that is no list of asked, unprotected names ends the step
/// before any removal, and the Console names what was wrong.
#[test]
fn an_answer_that_is_not_a_list_of_asked_unprotected_names_removes_nothing() {
    for (printed, text) in [
        ("bash", "protected bash"),
        ("linux", "linux"),
        ("--overwrite=*", "--overwrite"),
    ] {
        let mut runner = answering(&[printed]);

        let (ok, events) = remove(&mut runner, ["bash", "cowsay", "tree"]);

        assert!(!ok, "{printed}");
        assert_eq!(runner.ran, [QUERY_OF_THREE], "{printed}");
        assert!(said(&events, text), "{printed}: {events:?}");
    }
}

/// pacman refuses a removal another package requires and says why, which
/// fails the step with its words.
#[test]
fn a_removal_pacman_refuses_fails_the_step() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Failed {
            cause: Failure::ExitCode { code: 1_i32 },
            cleanup_failed: Vec::new(),
        }]),
        ..answering(&["cowsay"])
    };

    let (ok, events) = remove(&mut runner, ["cowsay", "tree"]);

    assert!(matches!(
        finished(&events).as_slice(),
        [(
            _,
            Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }
        )]
    ));
    assert_eq!(runner.ran, [QUERY, "pacman -Rn --noconfirm -- cowsay"]);
    assert!(!ok);
}
