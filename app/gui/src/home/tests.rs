use std::sync::Arc;

use toolbox_core::{ChangeSet, Snapshot};
use toolbox_system::CallError;

use super::{Home, HomeReaders};
use crate::command::{AppCommand, HomeCommand};
use crate::page::PageId;
use crate::pages::home::HomeProblem;
use crate::power::Power;
use crate::waking::{Wake, Waking};

/// logind says no to every restart.
#[derive(Debug)]
struct Refusing;

impl Power for Refusing {
    fn restart(&self, refused: Waking<CallError>) {
        refused.send(CallError::Denied);
    }
}

fn click(
    home: &mut Home,
    command: HomeCommand,
    changes: &ChangeSet,
) -> crate::page_service::Handled {
    home.handle(&AppCommand::Home(command), changes, &Snapshot::default())
}

#[test]
fn the_upgrade_is_queued_and_taken_back() {
    let mut home = Home::default();

    let queued = click(
        &mut home,
        HomeCommand::QueueUpgrade(true),
        &ChangeSet::default(),
    );

    let changes = queued.edit.unwrap();
    assert_eq!(changes.len(), 1);

    let dropped = click(&mut home, HomeCommand::QueueUpgrade(false), &changes);

    assert!(dropped.edit.is_some_and(|edit| edit.is_empty()));
}

#[test]
fn refresh_reads_home_and_asks_for_the_system() {
    let mut home = Home::default();

    let handled = click(&mut home, HomeCommand::Refresh, &ChangeSet::default());

    assert!(handled.taken);
    assert_eq!(handled.answers, [AppCommand::ReadSystem]);
}

#[test]
fn a_refused_restart_shows_until_dismissed() {
    let mut home = Home::new(
        HomeReaders::default(),
        Some(Arc::new(Refusing)),
        Wake::none(),
    );

    home.set_problem(Some(HomeProblem::QueueLost(2)));

    home.restart();
    assert_eq!(home.shown(&[], false).problem, None);

    home.take();
    assert_eq!(
        home.shown(&[], false).problem,
        Some(&HomeProblem::Restart(CallError::Denied))
    );

    let handled = click(
        &mut home,
        HomeCommand::DismissProblem,
        &ChangeSet::default(),
    );

    assert!(handled.taken);
    assert_eq!(home.shown(&[], false).problem, None);
}

/// A run, the installer and the Setup Assistant's end need App.
#[test]
fn the_clicks_that_need_app_are_left_to_it() {
    let mut home = Home::default();

    for command in [
        HomeCommand::Restart,
        HomeCommand::StartInstaller,
        HomeCommand::SetupFinished,
    ] {
        assert!(!click(&mut home, command, &ChangeSet::default()).taken);
    }
}

#[test]
fn without_a_check_there_is_no_garuda_update_and_no_tool() {
    let home = Home::default();

    assert!(!home.garuda_update());
    assert_eq!(home.tools(), []);
    assert_eq!(
        home.shown(&[PageId::Kernels], true).recent,
        [PageId::Kernels]
    );
}
