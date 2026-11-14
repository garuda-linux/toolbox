use toolbox_core::features::users::Refusal;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{HomeFolder, UserName};

use crate::command::{AppCommand, UserEdit, UsersCommand};
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

    users.handle(
        &AppCommand::Users(UsersCommand::DismissRefusal),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(users.refusal().is_none());
}
