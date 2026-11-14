use toolbox_core::features::users::Refusal;
use toolbox_core::features::users::run::{PageRun, RunKind};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{HomeFolder, UserName};

use crate::command::{AppCommand, UserEdit, UsersCommand};
use crate::page::PageId;
use crate::page_service::PageService as _;
use crate::users::Users;

fn remove(name: &str) -> AppCommand {
    AppCommand::Users(UsersCommand::Edit(UserEdit::Remove {
        user: UserName::new(name).unwrap(),
        home: HomeFolder::Keep,
    }))
}

/// Before the accounts are read, nothing about them is queued.
#[test]
fn a_removal_before_the_accounts_are_read_is_refused_with_the_name() {
    let mut users = Users::default();

    let handled = users.handle(&remove("ada"), &ChangeSet::default(), &Snapshot::default());

    assert!(handled.edit.is_none());
    let refused = users.refusal().unwrap();
    assert_eq!(refused.refusal, Refusal::Unavailable);
    assert_eq!(refused.subject, "ada");
}

#[test]
fn dismissing_takes_the_refusal_away() {
    let mut users = Users::default();
    users.handle(&remove("ada"), &ChangeSet::default(), &Snapshot::default());

    let other = users.receive(
        &AppCommand::DismissRefusal(PageId::Locales),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(!other.taken);
    assert!(users.refusal().is_some());

    let handled = users.receive(
        &AppCommand::DismissRefusal(PageId::Users),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert!(users.refusal().is_none());
}

/// A lock set outside the app shows in no probe, so the page reads the
/// flags and App the system.
#[test]
fn refresh_reads_the_flags_and_asks_for_the_system() {
    let mut users = Users::default();

    let handled = users.handle(
        &AppCommand::Users(UsersCommand::Refresh),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(handled.answers, [AppCommand::ReadSystem]);
}

#[test]
fn dismissing_the_run_forgets_it() {
    let mut users = Users::default();
    users.started(RunKind::Password, UserName::new("ada").unwrap());

    let handled = users.handle(
        &AppCommand::Users(UsersCommand::DismissRun),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(users.run(), &PageRun::None);
}
