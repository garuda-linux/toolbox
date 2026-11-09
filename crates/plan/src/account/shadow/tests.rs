use super::Caller;
use crate::ShadowLocks;
use crate::account::tests::{PASSWD, SHADOW};

fn names(locks: &ShadowLocks) -> Vec<&str> {
    locks.locks.iter().map(|lock| lock.user.as_str()).collect()
}

fn caller(uid: u32) -> Caller {
    Caller::new(uid).unwrap()
}

#[test]
fn the_answer_holds_the_people_and_not_the_system_accounts() {
    let locks = ShadowLocks::for_caller(SHADOW, PASSWD, caller(1000));

    assert_eq!(names(&locks), ["user", "guest", "nopass"]);
}

#[test]
fn root_and_system_accounts_cannot_ask() {
    for uid in [0, 1, 967, 999, 0xFFFE] {
        assert_eq!(Caller::new(uid), None, "{uid}");
    }

    assert!(Caller::new(1000).is_some());
    assert!(Caller::new(60001).is_some());
}

/// `nopass` moved to 61000 and `guest` to 61001, past the range of people.
/// The caller sees its own account and no other.
#[test]
fn a_caller_past_the_range_sees_its_own_account() {
    let passwd = PASSWD
        .replace("nopass:x:1002:1002:", "nopass:x:61000:61000:")
        .replace("guest:x:1001:1001:", "guest:x:61001:61001:");

    let locks = ShadowLocks::for_caller(SHADOW, &passwd, caller(61000));

    assert_eq!(names(&locks), ["user", "nopass"]);
}

/// The account `root` has `*`, no password at all, as `passwd -S` says L.
#[test]
fn a_star_counts_as_locked() {
    let locks = ShadowLocks::from_shadow(SHADOW);
    let root = locks.locks.iter().find(|lock| lock.user.as_str() == "root");

    assert_eq!(root.map(|lock| lock.locked), Some(true));
}
