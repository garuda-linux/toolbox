use std::collections::BTreeSet;

use super::super::rows::{group_choices, group_rows, user_rows};
use super::super::{add_group, remove_group, set_groups, set_locked};
use super::{groups, snapshot};
use crate::domains::Accounts;
use crate::test_support::{group_name, lock_flags, user_name};
use crate::{ChangeSet, Toggle};

#[test]
fn user_rows_filter_people_and_show_what_is_queued() {
    let mut snapshot = snapshot();

    // Joining wheel makes ada an admin, whose lock needs the flags.
    snapshot.insert(lock_flags(&[("ada", false), ("nico", false)]));
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    set_groups(&ada, &groups(&["makers", "wheel"]), &mut changes, &snapshot).unwrap();
    set_locked(&ada, Toggle::On, None, &mut changes, &snapshot).unwrap();

    let people = user_rows(accounts, &snapshot, &changes, true);
    let names: Vec<&str> = people.iter().map(|row| row.user.name.as_str()).collect();

    assert_eq!(names, ["ada", "nico"]);

    let row = people.first().unwrap();

    assert_eq!(row.groups, groups(&["ada", "audio", "makers"]));
    assert!(!row.current && row.editable);
    assert_eq!(row.queued.lock, Some(Toggle::On));
    assert_eq!(row.queued.joins, groups(&["wheel"]));
    assert_eq!(row.queued.leaves, groups(&["audio"]));
    assert!(people.last().unwrap().current);
    assert!(people.last().unwrap().queued.is_empty());

    let everyone = user_rows(accounts, &snapshot, &changes, false);

    assert_eq!(everyone.len(), 6);
    assert!(
        everyone
            .iter()
            .filter(|account| account.user.is_root())
            .all(|account| !account.editable)
    );
}

#[test]
fn group_rows_show_members_queued_groups_and_what_can_go() {
    let snapshot = snapshot();
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut changes = ChangeSet::default();

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();
    remove_group(&group_name("makers"), &mut changes, &snapshot).unwrap();

    let rows = group_rows(accounts, &changes);
    let find = |name: &str| rows.iter().find(|row| row.name.as_str() == name).unwrap();

    assert_eq!(rows.len(), 8);
    assert_eq!(find("crafts").gid, None);
    assert_eq!(find("crafts").queued, Some(Toggle::On));
    assert_eq!(find("makers").queued, Some(Toggle::Off));
    assert_eq!(find("wheel").queued, None);
    assert!(find("wheel").system && !find("makers").system);
    assert_eq!(
        find("makers").members,
        BTreeSet::from([user_name("ada"), user_name("nico")])
    );
    assert_eq!(find("nico").members, BTreeSet::from([user_name("nico")]));
}

#[test]
fn group_choices_show_the_state_after_the_queue() {
    let snapshot = snapshot();
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();
    remove_group(&group_name("makers"), &mut changes, &snapshot).unwrap();
    set_groups(&ada, &groups(&["crafts", "wheel"]), &mut changes, &snapshot).unwrap();

    let choices = group_choices(&ada, accounts, &changes);
    let state: Vec<(&str, bool, bool)> = choices
        .iter()
        .map(|choice| (choice.group.as_str(), choice.member, choice.primary))
        .collect();

    assert_eq!(
        state,
        [
            ("ada", true, true),
            ("audio", false, false),
            ("crafts", true, false),
            ("http", false, false),
            ("nico", false, false),
            ("root", false, false),
            ("wheel", true, false),
        ]
    );
}
