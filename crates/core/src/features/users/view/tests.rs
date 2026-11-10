use toolbox_plan::{FullName, ShellPath};

use super::{Shown, accounts_known, admin_group, choices, group_names, groups, shells, users};
use crate::domains::{Accounts, AvailableShells};
use crate::features::users::listing::{GroupOrder, UserOrder};
use crate::features::users::{add_group, set_full_name, set_shell};
use crate::test_support::{group, group_name, user, user_name};
use crate::{ChangeSet, Snapshot};

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

/// `ada` and `nico` as people, `http` as a service, `wheel` with `nico`.
fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![
            user("ada", 1001, 1001),
            user("http", 33, 33),
            user("nico", 1000, 1000),
        ],
        groups: vec![
            group("ada", 1001, &[]),
            group("http", 33, &[]),
            group("nico", 1000, &[]),
            group("wheel", 998, &["nico"]),
        ],
    });
    snapshot.insert(AvailableShells(vec![
        shell("/usr/bin/bash"),
        shell("/usr/bin/fish"),
    ]));
    snapshot
}

fn names(snapshot: &Snapshot, changes: &ChangeSet, shown: &Shown) -> Vec<String> {
    users(snapshot, changes, shown)
        .iter()
        .map(|row| row.user.name.to_string())
        .collect()
}

#[test]
fn the_page_opens_on_people_by_name_and_can_show_every_account() {
    let snapshot = snapshot();
    let changes = ChangeSet::default();

    assert_eq!(
        names(&snapshot, &changes, &Shown::people()),
        ["ada", "nico"]
    );

    let every = Shown {
        user_order: UserOrder::Id,
        ..Shown::default()
    };

    assert_eq!(names(&snapshot, &changes, &every), ["http", "nico", "ada"]);
}

#[test]
fn a_row_shows_and_counts_what_is_queued_for_it() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let nico = user_name("nico");
    set_shell(&nico, shell("/usr/bin/fish"), &mut changes, &snapshot).unwrap();
    set_full_name(
        &nico,
        FullName::new("Nico").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    let rows = users(&snapshot, &changes, &Shown::people());
    let row = rows.iter().find(|row| row.user.name == nico).unwrap();

    assert_eq!(row.shown_shell(), "/usr/bin/fish");
    assert_eq!(row.shown_full_name(), "Nico");
    assert_eq!(row.shown_home(), "/home/nico");
    assert_eq!(row.queued.count(), 2);
}

#[test]
fn groups_follow_the_search_and_a_queued_group_is_offered() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    add_group(&group_name("video"), &mut changes, &snapshot).unwrap();

    let shown = Shown {
        group_search: "WHE".to_owned(),
        group_order: GroupOrder::Members,
        ..Shown::default()
    };

    let listed: Vec<String> = groups(&snapshot, &changes, &shown)
        .iter()
        .map(|row| row.name.to_string())
        .collect();

    assert_eq!(listed, ["wheel"]);
    assert!(
        choices(&user_name("ada"), &snapshot, &changes)
            .iter()
            .any(|choice| choice.group == group_name("video") && choice.gid.is_none())
    );
}

#[test]
fn the_add_dialog_offers_wheel_as_administrator_and_lists_the_shells() {
    let snapshot = snapshot();
    let changes = ChangeSet::default();

    assert_eq!(admin_group(&snapshot, &changes), Some(&group_name("wheel")));
    assert_eq!(
        group_names(&snapshot, &changes, "wh"),
        [group_name("wheel")]
    );
    assert_eq!(shells(&snapshot).len(), 2);
}

#[test]
fn without_the_account_files_there_is_nothing_to_list() {
    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();

    assert!(!accounts_known(&snapshot));
    assert_eq!(users(&snapshot, &changes, &Shown::people()), []);
    assert_eq!(admin_group(&snapshot, &changes), None);
    assert_eq!(shells(&snapshot), []);
}

#[test]
fn the_lock_entry_takes_a_queued_wish_back_before_it_flips() {
    use super::LockAction;
    use crate::Toggle;

    assert_eq!(
        LockAction::of(Some(Toggle::On), Some(false)),
        Some(LockAction::UnqueueLock)
    );
    assert_eq!(
        LockAction::of(Some(Toggle::Off), None),
        Some(LockAction::UnqueueUnlock)
    );
    assert_eq!(LockAction::of(None, Some(true)), Some(LockAction::Unlock));
    assert_eq!(LockAction::of(None, Some(false)), Some(LockAction::Lock));
    assert_eq!(LockAction::of(None, None), None);
    assert!(LockAction::UnqueueLock.locked());
    assert!(!LockAction::Unlock.locked());
}
