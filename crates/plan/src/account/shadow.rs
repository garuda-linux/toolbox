//! Which lock flags the helper hands to whom. Any process of an active
//! session may ask without a password, so it learns about the accounts of
//! people and its own, never about root's or a service's.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use super::{LAST_USER_ID, ShadowLocks, passwd_entries};
use crate::UserName;
use crate::procedure::FIRST_USER_ID;

/// `nobody`, which no person logs in as.
const NOBODY: u32 = 0xFFFE;

const PEOPLE: RangeInclusive<u32> = FIRST_USER_ID..=LAST_USER_ID;

/// The account that asks, by the uid the kernel names for its socket.
/// Root and the system accounts never ask through the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caller(u32);

impl Caller {
    pub fn new(uid: u32) -> Option<Self> {
        (uid >= FIRST_USER_ID && uid != NOBODY).then_some(Self(uid))
    }

    pub const fn uid(self) -> u32 {
        self.0
    }
}

impl ShadowLocks {
    /// [`ShadowLocks::from_shadow`] for the accounts of people in `passwd`
    /// and the `caller`'s own.
    pub fn for_caller(shadow: &str, passwd: &str, caller: Caller) -> Self {
        let shown: BTreeSet<&str> = passwd_entries(passwd)
            .filter(|entry| PEOPLE.contains(&entry.uid) || entry.uid == caller.uid())
            .map(|entry| entry.name)
            .collect();

        let mut locks = Self::from_shadow(shadow);

        locks
            .locks
            .retain(|lock| shown.contains(UserName::as_str(&lock.user)));
        locks
    }
}
