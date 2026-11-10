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
    AccountLocks, Accounts, AvailableShells, CurrentUser, FullNames, LoginShells, RemovedUsers,
};
use crate::test_support::{NOBODY_ID, group, group_name, membership, user, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

/// `nico` runs the app and is the only admin. `ada` is a second person,
/// `http` a system account, `root` and `toor` have user id 0.
fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts {
        users: vec![
            user("ada", 1001, 1001),
            user("http", 33, 33),
            user("nico", 1000, 1000),
            user("nobody", NOBODY_ID, NOBODY_ID),
            user("root", 0, 0),
            user("toor", 0, 0),
        ],
        groups: vec![
            group("ada", 1001, &[]),
            group("audio", 996, &["ada"]),
            group("http", 33, &[]),
            group("makers", 1002, &["ada", "nico"]),
            group("nico", 1000, &[]),
            group("root", 0, &[]),
            group("wheel", 998, &["nico"]),
        ],
    });
    snapshot.insert(CurrentUser(user_name("nico")));
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
        &user_name("ada"),
        &groups(&["makers", "wheel"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(queued(&changes), ["groups ada audio", "groups ada wheel"]);
    assert_eq!(
        queued_memberships(&changes),
        [
            (membership("ada", "audio"), false),
            (membership("ada", "wheel"), true)
        ]
    );

    set_groups(
        &user_name("ada"),
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

    set_groups(&user_name("ada"), &groups(&[]), &mut changes, &snapshot).unwrap();

    assert_eq!(queued(&changes), ["groups ada audio", "groups ada makers"]);
}

#[test]
fn the_last_admin_stays_in_wheel() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_groups(
            &user_name("nico"),
            &groups(&["makers"]),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
    assert!(changes.is_empty(), "a refused dialog queues nothing");

    set_groups(
        &user_name("ada"),
        &groups(&["audio", "makers", "wheel"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_groups(
        &user_name("nico"),
        &groups(&["makers"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(queued(&changes), ["groups ada wheel", "groups nico wheel"]);

    assert_eq!(
        set_groups(
            &user_name("ada"),
            &groups(&["audio", "makers"]),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        remove_user(&user_name("ada"), HomeFolder::Keep, &mut changes, &snapshot),
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
            set_locked(&name, Toggle::On, Some(false), &mut changes, &snapshot),
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
fn the_current_user_cannot_lock_or_remove_itself() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_locked(
            &user_name("nico"),
            Toggle::On,
            Some(false),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::CurrentUser)
    );
    assert_eq!(
        remove_user(
            &user_name("nico"),
            HomeFolder::Keep,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::CurrentUser)
    );

    set_locked(
        &user_name("nico"),
        Toggle::Off,
        None,
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(queued(&changes), ["account-locks nico"]);
}

#[test]
fn a_lock_queues_once_and_a_second_click_takes_it_back() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    set_locked(&ada, Toggle::On, Some(false), &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<AccountLocks>(&ada), Some(&Toggle::On));

    set_locked(&ada, Toggle::On, Some(false), &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());

    set_locked(&ada, Toggle::On, Some(true), &mut changes, &snapshot).unwrap();
    set_locked(&ada, Toggle::Off, Some(false), &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());

    set_locked(&ada, Toggle::Off, None, &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<AccountLocks>(&ada), Some(&Toggle::Off));
    set_locked(&ada, Toggle::On, None, &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<AccountLocks>(&ada), Some(&Toggle::On));
}

#[test]
fn only_people_are_removed_and_their_other_changes_go_too() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

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
        &ada,
        &groups(&["crafts", "makers"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_locked(&ada, Toggle::On, None, &mut changes, &snapshot).unwrap();
    set_shell(
        &ada,
        ShellPath::new("/usr/bin/fish").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_full_name(&ada, FullName::new("Ada").unwrap(), &mut changes, &snapshot).unwrap();
    assert_eq!(changes.len(), 6);

    remove_user(&ada, HomeFolder::Delete, &mut changes, &snapshot).unwrap();

    assert_eq!(queued(&changes), ["new-groups crafts", "removed-users ada"]);
    assert_eq!(
        changes.wish::<RemovedUsers>(&ada),
        Some(&HomeFolder::Delete)
    );

    keep_user(&ada, &mut changes);
    assert_eq!(queued(&changes), ["new-groups crafts"]);
}

#[test]
fn shells_come_from_the_list_and_names_are_queued_as_a_diff() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    assert_eq!(
        set_shell(
            &ada,
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
        &ada,
        ShellPath::new("/usr/bin/fish").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    set_full_name(
        &ada,
        FullName::new("Ada L.").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert!(changes.wish::<LoginShells>(&ada).is_some());
    assert_eq!(
        changes.wish::<FullNames>(&ada).map(FullName::as_str),
        Some("Ada L.")
    );

    set_full_name(&ada, FullName::default(), &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<FullNames>(&ada), None);
}

#[test]
fn without_the_account_files_nothing_is_taken() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_groups(&user_name("ada"), &groups(&[]), &mut changes, &snapshot),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        add_group(&group_name("crafts"), &mut changes, &snapshot),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set_locked(&user_name("ada"), Toggle::On, None, &mut changes, &snapshot),
        Err(Refusal::Unavailable)
    );
    assert!(changes.is_empty());
}

#[test]
fn the_admin_groups_are_wheel_and_sudo() {
    assert_eq!(
        super::ADMIN_GROUPS
            .each_ref()
            .map(toolbox_plan::Name::as_str),
        ["wheel", "sudo"]
    );
}
