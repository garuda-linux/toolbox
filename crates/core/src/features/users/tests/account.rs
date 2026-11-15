use toolbox_plan::{FullName, HomeChange, HomeDirectory, ShellPath};

use super::super::Refusal;
use super::super::account::{AccountEdit, check_account, set_account};
use super::super::home::home_movable;
use super::{queued, snapshot};
use crate::ChangeSet;
use crate::test_support::user_name;

fn everything(home: &str) -> AccountEdit {
    AccountEdit {
        full_name: Some(FullName::new("Ada King").unwrap()),
        shell: Some(ShellPath::new("/usr/bin/fish").unwrap()),
        home: Some(HomeChange {
            home: HomeDirectory::new(home).unwrap(),
            move_files: true,
        }),
    }
}

#[test]
fn one_edit_queues_name_shell_and_home_together() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    set_account(
        &user_name("ada"),
        everything("/srv/ada"),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    let mut shown = queued(&changes);
    shown.sort();
    assert_eq!(
        shown,
        ["full-names ada", "home-folders ada", "login-shells ada"]
    );
}

/// The home folder of the own account cannot move, so the name typed in
/// the same dialog does not queue alone either.
#[test]
fn a_refused_part_queues_nothing_of_the_edit() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    for (user, edit, refusal) in [
        ("nico", everything("/srv/nico"), Refusal::CurrentUser),
        ("http", everything("/srv/http"), Refusal::OutsideUserRange),
        ("ada", everything("/home/nico"), Refusal::HomeTaken),
        (
            "ada",
            AccountEdit {
                shell: Some(ShellPath::new("/usr/bin/false").unwrap()),
                ..everything("/srv/ada")
            },
            Refusal::ShellNotListed,
        ),
        ("root", everything("/srv/root"), Refusal::Root),
    ] {
        assert_eq!(
            check_account(&user_name(user), &edit, &snapshot),
            Err(refusal),
            "{user}"
        );
        assert_eq!(
            set_account(&user_name(user), edit, &mut changes, &snapshot),
            Err(refusal),
            "{user}"
        );
    }

    assert!(changes.is_empty());
}

#[test]
fn the_dialog_learns_before_a_click_whether_the_home_may_move() {
    let snapshot = snapshot();

    assert_eq!(home_movable(&user_name("ada"), &snapshot), Ok(()));
    assert_eq!(
        home_movable(&user_name("nico"), &snapshot),
        Err(Refusal::CurrentUser)
    );
    assert_eq!(
        home_movable(&user_name("http"), &snapshot),
        Err(Refusal::OutsideUserRange)
    );

    let name_only = AccountEdit {
        full_name: Some(FullName::new("Nico J.").unwrap()),
        ..AccountEdit::default()
    };

    assert_eq!(
        check_account(&user_name("nico"), &name_only, &snapshot),
        Ok(())
    );
}
