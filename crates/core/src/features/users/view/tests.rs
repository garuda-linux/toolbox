use toolbox_plan::{FullName, ShellPath};

use super::{
    Shown, accounts_known, admin_group, choices, choices_holding, group_names, groups, shells,
    users,
};
use crate::domains::{Accounts, AvailableShells};
use crate::features::users::listing::GroupOrder;
use crate::features::users::{add_group, set_full_name, set_shell};
use crate::test_support::{group, group_name, user, user_name};
use crate::{ChangeSet, Snapshot};

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![
            user("grace", 1001, 1001),
            user("http", 33, 33),
            user("ada", 1000, 1000),
        ],
        groups: vec![
            group("grace", 1001, &[]),
            group("http", 33, &[]),
            group("ada", 1000, &[]),
            group("wheel", 998, &["ada"]),
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
fn the_page_opens_on_people_by_id_and_can_show_every_account() {
    let snapshot = snapshot();
    let changes = ChangeSet::default();

    assert_eq!(
        names(&snapshot, &changes, &Shown::people()),
        ["ada", "grace"]
    );

    let every = Shown::default();

    assert_eq!(names(&snapshot, &changes, &every), ["http", "ada", "grace"]);
}

#[test]
fn a_row_shows_and_counts_what_is_queued_for_it() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");
    set_shell(&ada, shell("/usr/bin/fish"), &mut changes, &snapshot).unwrap();
    set_full_name(&ada, FullName::new("Ada").unwrap(), &mut changes, &snapshot).unwrap();

    let rows = users(&snapshot, &changes, &Shown::people());
    let row = rows.iter().find(|row| row.user.name == ada).unwrap();

    assert_eq!(row.shown_shell(), "/usr/bin/fish");
    assert_eq!(row.shown_full_name(), "Ada");
    assert_eq!(row.shown_home(), "/home/ada");
    assert_eq!(row.queued.count(), 2);
}

#[test]
fn the_membership_search_keeps_the_groups_holding_it() {
    let snapshot = snapshot();
    let changes = ChangeSet::default();
    let every = choices(&user_name("grace"), &snapshot, &changes);

    let found: Vec<&str> = choices_holding(&every, "WHE")
        .iter()
        .map(|choice| choice.group.as_str())
        .collect();

    assert_eq!(found, ["wheel"]);
    assert_eq!(choices_holding(&every, "").len(), every.len());
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
        choices(&user_name("grace"), &snapshot, &changes)
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
        LockAction::of(Some(Toggle::On), Some(Toggle::Off)),
        Some(LockAction::UnqueueLock)
    );
    assert_eq!(
        LockAction::of(Some(Toggle::Off), None),
        Some(LockAction::UnqueueUnlock)
    );
    assert_eq!(
        LockAction::of(None, Some(Toggle::On)),
        Some(LockAction::Unlock)
    );
    assert_eq!(
        LockAction::of(None, Some(Toggle::Off)),
        Some(LockAction::Lock)
    );
    assert_eq!(LockAction::of(None, None), None);
    assert!(LockAction::UnqueueLock.locked());
    assert!(!LockAction::Unlock.locked());
}
