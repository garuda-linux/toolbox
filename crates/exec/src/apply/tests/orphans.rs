use std::collections::VecDeque;

use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{PackageName, RootAction, StepId, TargetList};

use super::{FakeRunner, finished, request, run};

fn lines(names: &[&str]) -> Vec<String> {
    names.iter().map(|&name| name.to_owned()).collect()
}

fn names_in(events: &[Event], text: &str) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Event::Output { step: _, data } if data.contains(text)))
        .count()
}

/// `efibootmgr` turned orphan once nothing installed needs it, yet the
/// protected list keeps it, so it stays out of `-Rn` and the Console
/// names it.
#[test]
fn a_protected_orphan_is_kept_and_named() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([lines(&["libfoo", "efibootmgr"])]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert_eq!(
        runner.ran,
        [
            "query: pacman -Qtdq",
            "pacman -Rn --noconfirm -- libfoo",
            "query: pacman -Qtdq"
        ]
    );
    assert_eq!(names_in(&events, "efibootmgr"), 1);
}

#[test]
fn only_protected_orphans_skip_the_command() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([lines(&["efibootmgr"])]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert_eq!(runner.ran, ["query: pacman -Qtdq"]);
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Skipped)]);
}

/// `grub-tools` was the only one needing `efibootmgr`. Once it is gone
/// the second round lists `efibootmgr`, and the filter keeps it, which
/// `-Rns` over the first round would have taken along unseen.
#[test]
fn a_dependency_an_orphan_freed_meets_the_filter_next_round() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([
            lines(&["grub-tools"]),
            lines(&["efibootmgr", "libbar"]),
            lines(&["efibootmgr"]),
        ]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert_eq!(
        runner.ran,
        [
            "query: pacman -Qtdq",
            "pacman -Rn --noconfirm -- grub-tools",
            "query: pacman -Qtdq",
            "pacman -Rn --noconfirm -- libbar",
            "query: pacman -Qtdq"
        ]
    );
    assert_eq!(names_in(&events, "efibootmgr"), 1);
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Succeeded)]);
}

/// pacman reported success and still lists the same orphans.
#[test]
fn orphans_that_stay_end_the_rounds() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([lines(&["libfoo"]), lines(&["libfoo"])]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran.len(), 3, "{:?}", runner.ran);
}

fn removing(names: &[&str]) -> RootAction {
    let packages: Vec<PackageName> = names
        .iter()
        .map(|&name| PackageName::new(name).unwrap())
        .collect();

    RootAction::RemovePackages {
        packages: TargetList::new(packages).unwrap(),
    }
}

#[test]
fn a_removal_that_takes_nothing_protected_runs() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([lines(&["gimp", "gegl"])]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![removing(&["gimp"])]),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert_eq!(
        runner.ran,
        [
            "query: pacman -Rs --print --print-format '%n' -- gimp",
            "pacman -Rns --noconfirm -- gimp"
        ]
    );
}

/// The request names `grub-tools` only, pacman would take `efibootmgr`.
#[test]
fn a_removal_that_takes_a_protected_package_along_is_refused() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([lines(&["grub-tools", "efibootmgr"])]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![removing(&["grub-tools"])]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(
        runner.ran,
        ["query: pacman -Rs --print --print-format '%n' -- grub-tools"]
    );
    assert_eq!(names_in(&events, "efibootmgr"), 1);
}

#[test]
fn a_preview_line_that_is_no_package_name_refuses_the_removal() {
    let mut runner = FakeRunner {
        queries: VecDeque::from([lines(&["checking dependencies..."])]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![removing(&["gimp"])]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran.len(), 1, "{:?}", runner.ran);
}
