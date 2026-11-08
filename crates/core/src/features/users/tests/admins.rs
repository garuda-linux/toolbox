//! Members of `wheel` or `sudo` may become root, so the last member of
//! either keeps a way to administer the system.

use toolbox_plan::HomeFolder;

use super::super::{
    Refusal, membership_allowed, recheck, recheck_locks, remove_user, set_groups, set_locked,
};
use super::{groups, queued, snapshot};
use crate::domains::{AccountLocks, Accounts, CurrentUser, Groups, RemovedUsers};
use crate::testing::{group, lock_flags, membership, user_name};
use crate::{ChangeSet, Snapshot, Toggle};

/// The fixture with `admins` as the group that grants sudo, holding
/// `members`. No other admin group exists.
pub(super) fn with_admins(admins: &str, members: &[&str]) -> Snapshot {
    let mut snapshot = snapshot();
    let mut accounts = snapshot.part::<Accounts>().unwrap().clone();

    accounts
        .groups
        .retain(|group| group.name.as_str() != "wheel");
    accounts.groups.push(group(admins, 27, members));
    snapshot.insert(accounts);
    snapshot
}

fn with_sudo_and_wheel() -> Snapshot {
    let mut snapshot = with_admins("sudo", &["nico"]);
    let mut accounts = snapshot.part::<Accounts>().unwrap().clone();

    accounts.groups.push(group("wheel", 998, &["nico"]));
    snapshot.insert(accounts);
    snapshot
}

#[test]
fn the_last_member_of_sudo_stays_an_admin() {
    let snapshot = with_admins("sudo", &["nico"]);
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
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn the_last_member_of_sudo_is_not_removed() {
    let mut snapshot = with_admins("sudo", &["ada"]);
    let mut changes = ChangeSet::default();

    snapshot.insert(CurrentUser(user_name("nico")));

    assert_eq!(
        remove_user(&user_name("ada"), HomeFolder::Keep, &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
}

#[test]
fn a_member_of_both_admin_groups_may_leave_one() {
    let snapshot = with_sudo_and_wheel();
    let mut changes = ChangeSet::default();

    set_groups(
        &user_name("nico"),
        &groups(&["makers", "sudo"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(queued(&changes), ["groups nico wheel"]);

    assert_eq!(
        set_groups(
            &user_name("nico"),
            &groups(&["makers"]),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
}

#[test]
fn one_membership_meets_the_same_guards() {
    let snapshot = with_admins("sudo", &["nico"]);
    let changes = ChangeSet::default();
    let leave = |user: &str, group: &str| {
        membership_allowed(&membership(user, group), Toggle::Off, &changes, &snapshot)
    };

    assert_eq!(leave("nico", "sudo"), Err(Refusal::LastAdmin));
    assert_eq!(leave("nico", "nico"), Err(Refusal::PrimaryGroup));
    assert_eq!(leave("root", "audio"), Err(Refusal::Root));
    assert_eq!(leave("ada", "audio"), Ok(()));
    assert_eq!(
        membership_allowed(
            &membership("ada", "docker"),
            Toggle::On,
            &changes,
            &snapshot
        ),
        Ok(()),
        "the package of a queued toggle creates the group"
    );
}

#[test]
fn a_member_of_both_admin_groups_may_leave_one_alone() {
    let snapshot = with_sudo_and_wheel();
    let mut changes = ChangeSet::default();

    assert_eq!(
        membership_allowed(
            &membership("nico", "wheel"),
            Toggle::Off,
            &changes,
            &snapshot
        ),
        Ok(())
    );

    changes.want::<Groups>(membership("nico", "wheel"), Toggle::Off, &snapshot);

    assert_eq!(
        membership_allowed(
            &membership("nico", "sudo"),
            Toggle::Off,
            &changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
}

/// Without `wheel` the app cannot tell who else may become root, so no
/// member of `sudo` loses the rights, even with a second member.
#[test]
fn without_wheel_no_admin_loses_the_rights() {
    let snapshot = with_admins("sudo", &["nico", "ada"]);
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    assert_eq!(
        remove_user(&ada, HomeFolder::Keep, &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        set_locked(&ada, Toggle::On, Some(false), &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        membership_allowed(&membership("ada", "sudo"), Toggle::Off, &changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        set_groups(&ada, &groups(&["audio"]), &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn without_wheel_a_user_outside_the_admin_groups_still_changes() {
    let snapshot = with_admins("sudo", &["nico"]);
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    assert_eq!(
        set_locked(&ada, Toggle::On, Some(false), &mut changes, &snapshot),
        Ok(())
    );
    assert_eq!(
        remove_user(&ada, HomeFolder::Keep, &mut changes, &snapshot),
        Ok(())
    );
    assert_eq!(queued(&changes), ["removed-users ada"]);
}

/// A queue that takes the rights of every admin already leaves none to
/// count, so the member of `wheel` loses nothing more.
#[test]
fn with_no_admin_left_after_the_queue_a_member_of_wheel_keeps_the_rights() {
    let mut snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let nico = user_name("nico");

    snapshot.insert(CurrentUser(user_name("ada")));
    changes.want::<Groups>(membership("nico", "wheel"), Toggle::Off, &snapshot);

    assert_eq!(
        membership_allowed(
            &membership("nico", "wheel"),
            Toggle::Off,
            &changes,
            &snapshot
        ),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        set_locked(&nico, Toggle::On, Some(false), &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(
        remove_user(&nico, HomeFolder::Keep, &mut changes, &snapshot),
        Err(Refusal::LastAdmin)
    );
    assert_eq!(changes.wish::<RemovedUsers>(&nico), None);
}

#[test]
fn without_wheel_a_restored_removal_or_lock_of_a_sudo_member_is_dropped() {
    let mut snapshot = with_admins("sudo", &["nico", "ada"]);
    let mut changes = ChangeSet::default();

    snapshot.insert(lock_flags(&[("nico", false), ("ada", false)]));
    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Keep, &snapshot);
    changes.want::<AccountLocks>(user_name("ada"), Toggle::On, &snapshot);

    assert_eq!(recheck(&mut changes, &snapshot), Some(1));
    assert_eq!(recheck_locks(&mut changes, &snapshot), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}

/// The fixture with `members` in `wheel` and `http` running the app, so
/// any admin may be locked as far as the own session goes. The helper
/// said that no password is locked.
pub(super) fn with_wheel(members: &[&str]) -> Snapshot {
    let mut snapshot = with_admins("wheel", members);
    let unlocked: Vec<(&str, bool)> = members.iter().map(|member| (*member, false)).collect();

    snapshot.insert(CurrentUser(user_name("http")));
    snapshot.insert(lock_flags(&unlocked));
    snapshot
}

/// A locked password keeps the membership but no admin could log in to
/// use it.
#[test]
fn the_last_admin_is_not_locked() {
    let snapshot = with_wheel(&["nico"]);
    let mut changes = ChangeSet::default();
    let nico = user_name("nico");

    assert_eq!(
        set_locked(&nico, Toggle::On, Some(false), &mut changes, &snapshot),
        Err(Refusal::LastAdminLock)
    );
    assert!(changes.is_empty(), "{changes:?}");
    assert_eq!(
        set_locked(&nico, Toggle::Off, Some(true), &mut changes, &snapshot),
        Ok(()),
        "unlocking is always allowed"
    );
}

#[test]
fn one_of_two_admins_may_be_locked() {
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
    assert_eq!(queued(&changes), ["account-locks nico"]);
}

/// Once the other admin is queued to leave `wheel`, the one left is the
/// last.
#[test]
fn an_admin_whose_peer_leaves_is_not_locked() {
    let snapshot = with_wheel(&["nico", "ada"]);
    let mut changes = ChangeSet::default();

    changes.want::<Groups>(membership("ada", "wheel"), Toggle::Off, &snapshot);

    assert_eq!(
        set_locked(
            &user_name("nico"),
            Toggle::On,
            Some(false),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastAdminLock)
    );
}

#[test]
fn a_restored_lock_of_the_last_admin_is_dropped() {
    let snapshot = with_wheel(&["nico"]);
    let mut changes = ChangeSet::default();

    changes.want::<AccountLocks>(user_name("nico"), Toggle::On, &snapshot);

    assert_eq!(recheck_locks(&mut changes, &snapshot), Some(1));
    assert!(changes.is_empty(), "{changes:?}");
}
