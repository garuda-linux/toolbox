//! An admin whose password is locked, now or by the queue, cannot log in
//! to use the rights, so it does not count as an admin who remains.

use serde_json::json;
use toolbox_plan::HomeFolder;

use super::super::{Refusal, membership_allowed, recheck, recheck_locks, remove_user, set_locked};
use super::admins::{with_admins, with_wheel};
use super::{queued, snapshot};
use crate::domains::{AccountLockFlags, AccountLocks, Accounts, CurrentUser, RemovedUsers};
use crate::test_support::{lock_flags, membership, restore_saved, user, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

/// `ada` and `grace` in `wheel`, `http` running the app, and the helper's
/// answer `flags` when it gave one.
fn two_admins(flags: Option<AccountLockFlags>) -> Snapshot {
    let mut snapshot = with_admins("wheel", &["ada", "grace"]);

    snapshot.insert(CurrentUser(user_name("http")));

    if let Some(flags) = flags {
        snapshot.insert(flags);
    }

    snapshot
}

#[test]
fn of_two_admins_only_one_is_queued_to_lock() {
    let snapshot = with_wheel(&["ada", "grace"]);
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_locked(
            &user_name("ada"),
            Toggle::On,
            Some(false),
            &mut changes,
            &snapshot
        ),
        Ok(())
    );
    assert_eq!(
        set_locked(
            &user_name("grace"),
            Toggle::On,
            Some(false),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdminLock)
    );
    assert_eq!(queued(&changes), ["account-locks ada"]);
}

#[test]
fn a_locked_admin_does_not_count_as_remaining() {
    let snapshot = two_admins(Some(lock_flags(&[("ada", false), ("grace", true)])));
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    assert_eq!(
        remove_user(&ada, HomeFolder::Keep, &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        set_locked(&ada, Toggle::On, Some(false), &mut changes, &snapshot),
        Err(Refusal::LastAdminLock)
    );
    assert_eq!(
        membership_allowed(
            &membership("ada", "wheel"),
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
    let snapshot = two_admins(Some(lock_flags(&[("ada", false), ("grace", true)])));
    let mut changes = ChangeSet::default();

    set_locked(
        &user_name("grace"),
        Toggle::Off,
        Some(true),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(
        remove_user(&user_name("ada"), HomeFolder::Keep, &mut changes, &snapshot),
        Ok(())
    );
}

#[test]
fn an_unknown_lock_state_refuses_only_a_lock_of_an_admin() {
    let snapshot = two_admins(None);
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_locked(&user_name("ada"), Toggle::On, None, &mut changes, &snapshot),
        Err(Refusal::LockStateUnknown)
    );
    let mut only_ada = with_admins("wheel", &["ada"]);
    only_ada.insert(CurrentUser(user_name("http")));
    assert_eq!(
        set_locked(
            &user_name("grace"),
            Toggle::On,
            None,
            &mut ChangeSet::default(),
            &only_ada
        ),
        Ok(()),
        "an account outside the admin groups needs no flags to be locked"
    );
    assert_eq!(
        set_locked(
            &user_name("ada"),
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
            &membership("grace", "wheel"),
            Toggle::Off,
            &changes,
            &snapshot
        ),
        Ok(())
    );
    assert_eq!(
        remove_user(
            &user_name("grace"),
            HomeFolder::Keep,
            &mut changes,
            &snapshot
        ),
        Ok(()),
        "the admins are counted by membership"
    );
}

/// The helper refuses the same, so the queue never holds what fails.
#[test]
fn only_accounts_with_an_id_from_1000_to_60000_are_locked() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    for name in ["nobody", "http"] {
        assert_eq!(
            set_locked(
                &user_name(name),
                Toggle::On,
                Some(false),
                &mut changes,
                &snapshot
            ),
            Err(Refusal::OutsideUserRange),
            "{name}"
        );
        assert_eq!(
            set_locked(
                &user_name(name),
                Toggle::Off,
                Some(true),
                &mut changes,
                &snapshot
            ),
            Ok(()),
            "unlocking {name} is allowed"
        );
    }
}

/// Both ends of the range lock, the ids on either side of it do not.
#[test]
fn the_range_includes_1000_and_60000_and_no_id_beside_them() {
    let mut snapshot = snapshot();
    snapshot.insert(Accounts {
        users: vec![
            user("before", 999, 999),
            user("first", 1000, 1000),
            user("last", 60000, 60000),
            user("after", 60001, 60001),
        ],
        groups: Vec::new(),
    });
    let mut changes = ChangeSet::default();

    for (name, allowed) in [
        ("before", false),
        ("first", true),
        ("last", true),
        ("after", false),
    ] {
        let expected = if allowed {
            Ok(())
        } else {
            Err(Refusal::OutsideUserRange)
        };

        assert_eq!(
            set_locked(
                &user_name(name),
                Toggle::On,
                Some(false),
                &mut changes,
                &snapshot
            ),
            expected,
            "{name}"
        );
    }
}

fn restored_lock(user: &str) -> ChangeSet {
    restore_saved(&[("account-locks", json!(user), json!("on"))])
}

#[test]
fn a_restored_lock_of_the_last_unlocked_admin_waits_for_the_flags() {
    let mut changes = restored_lock("ada");

    assert_eq!(recheck_locks(&mut changes, &two_admins(None)), None);
    assert_eq!(changes.len(), 1, "the lock state is not known yet");

    let known = two_admins(Some(lock_flags(&[("ada", false), ("grace", true)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn of_two_restored_locks_of_the_admins_one_stays() {
    let mut changes = restored_lock("ada");
    let snapshot = two_admins(Some(lock_flags(&[("ada", false), ("grace", false)])));

    changes.want::<AccountLocks>(user_name("grace"), Toggle::On, &snapshot);

    assert_eq!(recheck_locks(&mut changes, &snapshot), Some(1));
    assert_eq!(changes.len(), 1, "{changes:?}");
}

/// Without the flags a removal or leave is counted by membership, so it
/// is checked again once they tell that the other admin is locked.
#[test]
fn a_restored_removal_of_the_last_unlocked_admin_is_dropped_once_the_flags_arrive() {
    let mut changes = restore_saved(&[("removed-users", json!("ada"), json!("keep"))]);

    assert_eq!(recheck(&mut changes, &two_admins(None)), Some(0));

    let known = two_admins(Some(lock_flags(&[("ada", false), ("grace", true)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn a_restored_leave_of_the_last_unlocked_admin_is_dropped_once_the_flags_arrive() {
    let mut changes = restore_saved(&[(
        "groups",
        json!({ "user": "ada", "group": "wheel" }),
        json!("off"),
    )]);

    assert_eq!(recheck(&mut changes, &two_admins(None)), Some(0));

    let known = two_admins(Some(lock_flags(&[("ada", false), ("grace", true)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

/// A lock is easily undone and a removal is not, so the removal goes.
#[test]
fn of_a_restored_lock_and_removal_of_two_admins_the_lock_stays() {
    let mut changes = restored_lock("ada");
    let known = two_admins(Some(lock_flags(&[("ada", false), ("grace", false)])));

    changes.want::<RemovedUsers>(user_name("grace"), HomeFolder::Keep, &known);

    assert_eq!(recheck_locks(&mut changes, &known), Some(1));
    assert_eq!(queued(&changes), ["account-locks ada"]);
}

#[test]
fn a_restored_removal_of_one_of_two_unlocked_admins_stays() {
    let mut changes = restore_saved(&[("removed-users", json!("ada"), json!("keep"))]);
    let known = two_admins(Some(lock_flags(&[("ada", false), ("grace", false)])));

    assert_eq!(recheck_locks(&mut changes, &known), Some(0));
    assert_eq!(queued(&changes), ["removed-users ada"]);
}
