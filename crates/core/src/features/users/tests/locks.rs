//! An admin whose password is locked, now or by the queue, cannot log in
//! to use the rights, so it does not count as an admin who remains.

use serde_json::json;
use toolbox_plan::HomeFolder;

use super::super::{Refusal, membership_allowed, recheck, recheck_locks, remove_user, set_locked};
use super::admins::{with_admins, with_wheel};
use super::queued;
use crate::domains::{AccountLockFlags, AccountLocks, CurrentUser, RemovedUsers};
use crate::testing::{lock_flags, membership, restore_saved, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

/// `nico` and `ada` in `wheel`, `http` running the app, and the helper's
/// answer `flags` when it gave one.
fn two_admins(flags: Option<AccountLockFlags>) -> Snapshot {
    let mut snapshot = with_admins("wheel", &["nico", "ada"]);

    snapshot.insert(CurrentUser(user_name("http")));

    if let Some(flags) = flags {
        snapshot.insert(flags);
    }

    snapshot
}

#[test]
fn of_two_admins_only_one_is_queued_to_lock() {
    let snapshot = with_wheel(&["nico", "ada"]);
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_locked(
            &user_name("nico"),
            Toggle::On,
            Some(false),
            &mut changes,
            &snapshot
        ),
        Ok(())
    );
    assert_eq!(
        set_locked(
            &user_name("ada"),
            Toggle::On,
            Some(false),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdminLock)
    );
    assert_eq!(queued(&changes), ["account-locks nico"]);
}

#[test]
fn a_locked_admin_does_not_count_as_remaining() {
    let snapshot = two_admins(Some(lock_flags(&[("nico", false), ("ada", true)])));
    let mut changes = ChangeSet::default();
    let nico = user_name("nico");

    assert_eq!(
        remove_user(&nico, HomeFolder::Keep, &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        set_locked(&nico, Toggle::On, Some(false), &mut changes, &snapshot),
        Err(Refusal::LastAdminLock)
    );
    assert_eq!(
        membership_allowed(
            &membership("nico", "wheel"),
            Toggle::Off,
            &changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
    assert!(changes.is_empty(), "{changes:?}");
}

/// Once the queue unlocks the other admin, it counts again.
#[test]
fn a_queued_unlock_makes_a_locked_admin_count_again() {
    let snapshot = two_admins(Some(lock_flags(&[("nico", false), ("ada", true)])));
    let mut changes = ChangeSet::default();

    set_locked(
        &user_name("ada"),
        Toggle::Off,
        Some(true),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(
        remove_user(
            &user_name("nico"),
            HomeFolder::Keep,
            &mut changes,
            &snapshot
        ),
        Ok(())
    );
}

#[test]
fn an_unknown_lock_state_refuses_only_a_lock_of_an_admin() {
    let snapshot = two_admins(None);
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_locked(
            &user_name("nico"),
            Toggle::On,
            None,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LockStateUnknown)
    );
    assert_eq!(
        set_locked(
            &user_name("nobody"),
            Toggle::On,
            None,
            &mut changes,
            &snapshot
        ),
        Ok(()),
        "an account outside the admin groups needs no flags to be locked"
    );
    assert_eq!(
        set_locked(
            &user_name("nico"),
            Toggle::Off,
            None,
            &mut changes,
            &snapshot
        ),
        Ok(()),
        "unlocking is always allowed"
    );
    assert_eq!(
        membership_allowed(
            &membership("ada", "wheel"),
            Toggle::Off,
            &changes,
            &snapshot
        ),
        Ok(())
    );
    assert_eq!(
        remove_user(&user_name("ada"), HomeFolder::Keep, &mut changes, &snapshot),
        Ok(()),
        "the admins are counted by membership"
    );
}

fn restored_lock(user: &str) -> ChangeSet {
    restore_saved(&[("account-locks", json!(user), json!("on"))])
}

#[test]
fn a_restored_lock_of_the_last_unlocked_admin_waits_for_the_flags() {
    let mut changes = restored_lock("nico");

    assert_eq!(recheck_locks(&mut changes, &two_admins(None)), None);
    assert_eq!(changes.len(), 1, "the lock state is not known yet");

    let known = two_admins(Some(lock_flags(&[("nico", false), ("ada", true)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn of_two_restored_locks_of_the_admins_one_stays() {
    let mut changes = restored_lock("nico");
    let snapshot = two_admins(Some(lock_flags(&[("nico", false), ("ada", false)])));

    changes.want::<AccountLocks>(user_name("ada"), Toggle::On, &snapshot);

    assert_eq!(recheck_locks(&mut changes, &snapshot), Some(1));
    assert_eq!(changes.len(), 1, "{changes:?}");
}

/// Without the flags a removal or leave is counted by membership, so it
/// is checked again once they tell that the other admin is locked.
#[test]
fn a_restored_removal_of_the_last_unlocked_admin_is_dropped_once_the_flags_arrive() {
    let mut changes = restore_saved(&[("removed-users", json!("nico"), json!("keep"))]);

    assert_eq!(recheck(&mut changes, &two_admins(None)), Some(0));

    let known = two_admins(Some(lock_flags(&[("nico", false), ("ada", true)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn a_restored_leave_of_the_last_unlocked_admin_is_dropped_once_the_flags_arrive() {
    let mut changes = restore_saved(&[(
        "groups",
        json!({ "user": "nico", "group": "wheel" }),
        json!("off"),
    )]);

    assert_eq!(recheck(&mut changes, &two_admins(None)), Some(0));

    let known = two_admins(Some(lock_flags(&[("nico", false), ("ada", true)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

/// A lock is easily undone and a removal is not, so the removal goes.
#[test]
fn of_a_restored_lock_and_removal_of_two_admins_the_lock_stays() {
    let mut changes = restored_lock("nico");
    let known = two_admins(Some(lock_flags(&[("nico", false), ("ada", false)])));

    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Keep, &known);

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert_eq!(queued(&changes), ["account-locks nico"]);
}

#[test]
fn a_restored_removal_of_one_of_two_unlocked_admins_stays() {
    let mut changes = restore_saved(&[("removed-users", json!("nico"), json!("keep"))]);
    let known = two_admins(Some(lock_flags(&[("nico", false), ("ada", false)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(0));
    assert_eq!(queued(&changes), ["removed-users nico"]);
}
