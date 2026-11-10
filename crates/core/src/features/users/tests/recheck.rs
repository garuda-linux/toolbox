//! A queue restored from disk never met the page's guards: the recheck
//! applies them once the system is known.

use serde_json::json;

use super::super::{recheck, recheck_locks};
use super::admins::with_admins;
use super::{queued, snapshot};
use crate::Snapshot;
use crate::domains::CurrentUser;
use crate::test_support::{lock_flags, restore_saved, user_name};

/// `ada` runs the app, `nico` stays the only admin.
fn ada_runs_the_app() -> Snapshot {
    let mut snapshot = snapshot();

    snapshot.insert(CurrentUser(user_name("ada")));
    snapshot
}

#[test]
fn a_restored_removal_of_the_own_account_is_dropped() {
    let mut changes = restore_saved(&[
        ("removed-users", json!("nico"), json!("keep")),
        ("removed-users", json!("ada"), json!("delete")),
    ]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert_eq!(queued(&changes), ["removed-users ada"]);
}

#[test]
fn a_restored_removal_of_the_last_admin_is_dropped() {
    let mut changes = restore_saved(&[("removed-users", json!("nico"), json!("keep"))]);

    assert_eq!(recheck(&mut changes, &ada_runs_the_app()), Some(1));
    assert_eq!(queued(&changes), Vec::<String>::new());
}

#[test]
fn a_restored_removal_of_root_or_a_system_account_is_dropped() {
    let mut changes = restore_saved(&[
        ("removed-users", json!("root"), json!("keep")),
        ("removed-users", json!("http"), json!("keep")),
    ]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(2));
    assert!(changes.is_empty());
}

#[test]
fn the_last_admin_does_not_leave_wheel_through_a_restored_queue() {
    let mut changes = restore_saved(&[
        (
            "groups",
            json!({ "user": "nico", "group": "wheel" }),
            json!("off"),
        ),
        (
            "groups",
            json!({ "user": "nico", "group": "makers" }),
            json!("off"),
        ),
    ]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert_eq!(queued(&changes), ["groups nico makers"]);
}

#[test]
fn the_last_admin_does_not_leave_sudo_through_a_restored_queue() {
    let mut changes = restore_saved(&[(
        "groups",
        json!({ "user": "nico", "group": "sudo" }),
        json!("off"),
    )]);

    assert_eq!(
        recheck(&mut changes, &with_admins("sudo", &["nico"])),
        Some(1)
    );
    assert_eq!(queued(&changes), Vec::<String>::new());
}

#[test]
fn a_restored_lock_of_the_own_account_is_dropped() {
    let mut changes = restore_saved(&[
        ("account-locks", json!("nico"), json!("on")),
        ("account-locks", json!("ada"), json!("on")),
    ]);

    let mut snapshot = snapshot();

    snapshot.insert(lock_flags(&[("nico", false), ("ada", false)]));

    assert_eq!(
        recheck(&mut changes, &snapshot),
        Some(0),
        "locks wait for the flags"
    );
    assert_eq!(recheck_locks(&mut changes, &snapshot), Some(1));
    assert_eq!(queued(&changes), ["account-locks ada"]);
}

#[test]
fn an_allowed_restored_removal_keeps_its_wish() {
    let mut changes = restore_saved(&[("removed-users", json!("ada"), json!("delete"))]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(0));
    assert_eq!(
        changes.to_saved().changes[0].wish,
        json!("delete"),
        "the home folder choice survives the recheck"
    );
}

#[test]
fn without_the_account_files_the_restored_queue_stays() {
    let mut changes = restore_saved(&[("removed-users", json!("nico"), json!("keep"))]);

    assert_eq!(recheck(&mut changes, &Snapshot::default()), None);
    assert_eq!(changes.len(), 1);
}

#[test]
fn without_the_current_user_the_restored_queue_waits() {
    use crate::domains::Accounts;

    let full = snapshot();
    let accounts = full.part::<Accounts>().unwrap().clone();
    let mut without_current = Snapshot::default();

    without_current.insert(accounts);
    let mut changes = restore_saved(&[("removed-users", json!("nico"), json!("keep"))]);

    assert_eq!(recheck(&mut changes, &without_current), None);
    assert_eq!(changes.len(), 1);

    assert_eq!(recheck(&mut changes, &full), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_home_change_of_the_own_account_is_dropped() {
    let mut changes = restore_saved(&[
        (
            "home-folders",
            json!("ada"),
            json!({ "home": "/srv/ada", "move_files": false }),
        ),
        (
            "home-folders",
            json!("nico"),
            json!({ "home": "/srv/nico", "move_files": true }),
        ),
    ]);

    assert_eq!(recheck(&mut changes, &ada_runs_the_app()), Some(1));
    assert_eq!(queued(&changes), ["home-folders nico"]);
}

#[test]
fn a_restored_home_folder_another_account_has_is_dropped() {
    let mut changes = restore_saved(&[(
        "home-folders",
        json!("ada"),
        json!({ "home": "/home/nico", "move_files": false }),
    )]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_change_of_roots_account_is_dropped() {
    let mut changes = restore_saved(&[
        ("login-shells", json!("root"), json!("/usr/bin/fish")),
        ("full-names", json!("root"), json!("System")),
        (
            "home-folders",
            json!("root"),
            json!({ "home": "/srv/root2", "move_files": false }),
        ),
    ]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(3));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_change_of_an_unknown_account_is_dropped() {
    let mut changes = restore_saved(&[
        ("login-shells", json!("ghost"), json!("/usr/bin/fish")),
        ("full-names", json!("ghost"), json!("Ghost")),
        (
            "home-folders",
            json!("ghost"),
            json!({ "home": "/srv/ghost", "move_files": false }),
        ),
    ]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(3));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_shell_that_etc_shells_does_not_list_is_dropped() {
    let mut changes = restore_saved(&[
        ("login-shells", json!("ada"), json!("/usr/bin/fish")),
        ("login-shells", json!("nico"), json!("/usr/bin/zsh")),
    ]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert_eq!(queued(&changes), ["login-shells ada"]);
}
