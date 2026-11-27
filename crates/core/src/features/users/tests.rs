mod account;
mod admins;
mod groups;
mod home;
mod listing;
mod locks;
mod now;
mod recheck;
mod recheck_groups;
mod rows;

use std::collections::BTreeSet;

use toolbox_plan::{FullName, GroupName, HomeFolder, ShellPath};

use super::rows::queued_memberships;
use super::{
    Refusal, add_group, keep_user, remove_user, set_full_name, set_groups, set_locked, set_shell,
};
use crate::domains::{
    Accounts, AvailableShells, CurrentUser, FullNames, LoginShells, RemovedUsers,
};
use crate::test_support::{NOBODY_ID, group, group_name, membership, user, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

/// `ada` runs the app and is the only admin. `grace` is a second person,
/// `http` a system account, `root` and `toor` have user id 0.
fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts {
        users: vec![
            user("grace", 1001, 1001),
            user("http", 33, 33),
            user("ada", 1000, 1000),
            user("nobody", NOBODY_ID, NOBODY_ID),
            user("root", 0, 0),
            user("toor", 0, 0),
        ],
        groups: vec![
            group("grace", 1001, &[]),
            group("audio", 996, &["grace"]),
            group("http", 33, &[]),
            group("makers", 1002, &["grace", "ada"]),
            group("ada", 1000, &[]),
            group("root", 0, &[]),
            group("wheel", 998, &["ada"]),
        ],
    });
    snapshot.insert(CurrentUser(user_name("ada")));
    snapshot.insert(AvailableShells(vec![
        ShellPath::new("/usr/bin/bash").unwrap(),
        ShellPath::new("/usr/bin/fish").unwrap(),
    ]));
    snapshot
}

fn groups(names: &[&str]) -> BTreeSet<GroupName> {
    names.iter().map(|name| group_name(name)).collect()
}

fn queued(changes: &ChangeSet) -> Vec<String> {
    changes
        .views()
        .into_iter()
        .map(|view| format!("{} {}", view.domain, view.key))
        .collect()
}

#[test]
fn the_group_dialog_queues_only_the_differences() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    set_groups(
        &user_name("grace"),
        &groups(&["makers", "wheel"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(
        queued(&changes),
        ["groups grace audio", "groups grace wheel"]
    );
    assert_eq!(
        queued_memberships(&changes),
        [
            (membership("grace", "audio"), false),
            (membership("grace", "wheel"), true)
        ]
    );

    set_groups(
        &user_name("grace"),
        &groups(&["audio", "makers"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert!(changes.is_empty());
}

#[test]
fn the_primary_group_is_never_left() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    set_groups(&user_name("grace"), &groups(&[]), &mut changes, &snapshot).unwrap();

    assert_eq!(
        queued(&changes),
        ["groups grace audio", "groups grace makers"]
    );
}

#[test]
fn the_last_admin_stays_in_wheel() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_groups(
            &user_name("ada"),
            &groups(&["makers"]),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
    assert!(changes.is_empty(), "a refused dialog queues nothing");

    set_groups(
        &user_name("grace"),
        &groups(&["audio", "makers", "wheel"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_groups(
        &user_name("ada"),
        &groups(&["makers"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(queued(&changes), ["groups ada wheel", "groups grace wheel"]);

    assert_eq!(
        set_groups(
            &user_name("grace"),
            &groups(&["audio", "makers"]),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        remove_user(
            &user_name("grace"),
            HomeFolder::Keep,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
}

#[test]
fn root_accounts_are_not_changed_at_all() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    for name in ["root", "toor"] {
        let name = user_name(name);

        assert_eq!(
            set_groups(&name, &groups(&["wheel"]), &mut changes, &snapshot),
            Err(Refusal::Root)
        );
        assert_eq!(
            set_locked(
                &name,
                Toggle::On,
                Some(Toggle::Off),
                &mut changes,
                &snapshot
            ),
            Err(Refusal::Root)
        );
        assert_eq!(
            remove_user(&name, HomeFolder::Keep, &mut changes, &snapshot),
            Err(Refusal::Root)
        );
        assert_eq!(
            set_shell(
                &name,
                ShellPath::new("/usr/bin/fish").unwrap(),
                &mut changes,
                &snapshot
            ),
            Err(Refusal::Root)
        );
        assert_eq!(
            set_full_name(&name, FullName::default(), &mut changes, &snapshot),
            Err(Refusal::Root)
        );
    }

    assert!(changes.is_empty());
}

#[test]
fn only_people_are_removed_and_their_other_changes_go_too() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let grace = user_name("grace");

    for (name, refusal) in [
        ("http", Refusal::SystemAccount),
        ("nobody", Refusal::SystemAccount),
        ("ghost", Refusal::UnknownUser),
    ] {
        assert_eq!(
            remove_user(&user_name(name), HomeFolder::Keep, &mut changes, &snapshot),
            Err(refusal),
            "{name}"
        );
    }

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();
    set_groups(
        &grace,
        &groups(&["crafts", "makers"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_locked(&grace, Toggle::On, None, &mut changes, &snapshot).unwrap();
    set_shell(
        &grace,
        ShellPath::new("/usr/bin/fish").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_full_name(
        &grace,
        FullName::new("Grace").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(changes.len(), 6);

    remove_user(&grace, HomeFolder::Delete, &mut changes, &snapshot).unwrap();

    assert_eq!(
        queued(&changes),
        ["new-groups crafts", "removed-users grace"]
    );
    assert_eq!(
        changes.wish::<RemovedUsers>(&grace),
        Some(&HomeFolder::Delete)
    );

    keep_user(&grace, &mut changes);
    assert_eq!(queued(&changes), ["new-groups crafts"]);
}

#[test]
fn shells_come_from_the_list_and_names_are_queued_as_a_diff() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let grace = user_name("grace");

    assert_eq!(
        set_shell(
            &grace,
            ShellPath::new("/usr/bin/false").unwrap(),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::ShellNotListed)
    );
    assert_eq!(
        set_shell(
            &user_name("ghost"),
            ShellPath::new("/usr/bin/fish").unwrap(),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::UnknownUser)
    );

    set_shell(
        &grace,
        ShellPath::new("/usr/bin/fish").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_full_name(
        &grace,
        FullName::new("Grace L.").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert!(changes.wish::<LoginShells>(&grace).is_some());
    assert_eq!(
        changes.wish::<FullNames>(&grace).map(FullName::as_str),
        Some("Grace L.")
    );

    set_full_name(&grace, FullName::default(), &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<FullNames>(&grace), None);
}

#[test]
fn without_the_account_files_nothing_is_taken() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_groups(&user_name("grace"), &groups(&[]), &mut changes, &snapshot),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        add_group(&group_name("crafts"), &mut changes, &snapshot),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set_locked(
            &user_name("grace"),
            Toggle::On,
            None,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::Unavailable)
    );
    assert!(changes.is_empty());
}
