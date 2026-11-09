use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{RootAction, StepId};

use super::{FakeRunner, finished, request, run};

/// `efibootmgr` turned orphan once nothing installed needs it, yet the
/// protected list keeps it, so it stays out of `-Rns` and the Console
/// names it.
#[test]
fn a_protected_orphan_is_kept_and_named() {
    let mut runner = FakeRunner {
        query_output: vec!["libfoo".to_owned(), "efibootmgr".to_owned()],
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
        ["query: pacman -Qtdq", "pacman -Rns --noconfirm -- libfoo"]
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Output { step: _, data } if data.contains("efibootmgr")
    )));
}

#[test]
fn only_protected_orphans_skip_the_command() {
    let mut runner = FakeRunner {
        query_output: vec!["efibootmgr".to_owned()],
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
