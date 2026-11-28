use toolbox_plan::{FullName, HomeChange, HomeDirectory, ShellPath};

use super::super::Refusal;
use super::super::account::{AccountEdit, Changes, Fields, check_account, set_account};
use super::super::home::home_movable;
use super::{queued, snapshot};
use crate::ChangeSet;
use crate::test_support::user_name;

fn everything(home: &str) -> AccountEdit {
    AccountEdit {
        full_name: Some(FullName::new("Grace King").unwrap()),
        shell: Some(ShellPath::new("/usr/bin/fish").unwrap()),
        home: Some(HomeChange {
            home: HomeDirectory::new(home).unwrap(),
            move_files: true,
        }),
    }
}

fn fields(home: &str, move_files: bool) -> Fields {
    Fields {
        full_name: "Grace King".to_owned(),
        shell: "/usr/bin/fish".to_owned(),
        home: home.to_owned(),
        move_files,
    }
}

#[test]
fn the_move_flag_alone_changes_no_home_folder() {
    let start = fields("/home/grace", false);
    let now = fields("/home/grace", true);

    let changes = Changes::between(&start, &now, "/home/grace");

    assert!(!changes.any());
    assert!(changes.home.is_none());
}

#[test]
fn a_moved_folder_takes_the_move_flag_along() {
    let start = fields("/home/grace", false);
    let now = fields("/srv/grace", true);

    let changes = Changes::between(&start, &now, "/home/grace");

    assert_eq!(
        changes.edit(true).and_then(|edit| edit.home),
        Some(HomeChange {
            home: HomeDirectory::new("/srv/grace").unwrap(),
            move_files: true,
        })
    );
}

#[test]
fn one_edit_queues_name_shell_and_home_together() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    set_account(
        &user_name("grace"),
        everything("/srv/grace"),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    let mut shown = queued(&changes);
    shown.sort();
    assert_eq!(
        shown,
        [
            "full-names grace",
            "home-folders grace",
            "login-shells grace"
        ]
    );
}

/// The home folder of the own account cannot move, so the name typed in
/// the same dialog does not queue alone either.
#[test]
fn a_refused_part_queues_nothing_of_the_edit() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    for (user, edit, refusal) in [
        ("ada", everything("/srv/ada"), Refusal::CurrentUser),
        ("http", everything("/srv/http"), Refusal::OutsideUserRange),
        ("grace", everything("/home/ada"), Refusal::HomeTaken),
        (
            "grace",
            AccountEdit {
                shell: Some(ShellPath::new("/usr/bin/false").unwrap()),
                ..everything("/srv/grace")
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

    assert_eq!(home_movable(&user_name("grace"), &snapshot), Ok(()));
    assert_eq!(
        home_movable(&user_name("ada"), &snapshot),
        Err(Refusal::CurrentUser)
    );
    assert_eq!(
        home_movable(&user_name("http"), &snapshot),
        Err(Refusal::OutsideUserRange)
    );

    let name_only = AccountEdit {
        full_name: Some(FullName::new("Ada L.").unwrap()),
        ..AccountEdit::default()
    };

    assert_eq!(
        check_account(&user_name("ada"), &name_only, &snapshot),
        Ok(())
    );
}
