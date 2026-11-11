//! What the Users page knows of the locked passwords: not asked yet,
//! asking, or the helper's answer. `toolbox-system`'s `LockSource` reads
//! them.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use toolbox_plan::UserName;
use toolbox_state::accounts::AccountLockFlags;
use toolbox_system::LockProblem;
use toolbox_system::pkexec::Refusal;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Locks {
    /// The page was not opened yet, so nothing asked the helper.
    #[default]
    NotAsked,
    Loading,
    Known(BTreeMap<UserName, bool>),
    Refused(Refusal),
    Unknown,
}

impl Locks {
    pub fn of_answer(answer: Result<BTreeMap<UserName, bool>, LockProblem>) -> Self {
        match answer {
            Ok(flags) => Self::Known(flags),
            Err(LockProblem::Refused(refusal)) => Self::Refused(refusal),
            Err(LockProblem::Failed(_)) => Self::Unknown,
        }
    }

    pub fn of(&self, user: &UserName) -> Option<bool> {
        match self {
            Self::Known(flags) => flags.get(user).copied(),
            Self::NotAsked | Self::Loading | Self::Refused(_) | Self::Unknown => None,
        }
    }

    /// The snapshot part of a read that worked. `None` leaves the lock
    /// state unknown to the guards.
    pub fn flags(&self) -> Option<AccountLockFlags> {
        match self {
            Self::Known(flags) => Some(AccountLockFlags(flags.clone())),
            Self::NotAsked | Self::Loading | Self::Refused(_) | Self::Unknown => None,
        }
    }

    pub const fn code(&self) -> &'static str {
        match self {
            Self::NotAsked | Self::Loading => "loading",
            Self::Known(_) => "known",
            Self::Refused(refusal) => refusal.code(),
            Self::Unknown => "unknown",
        }
    }
}
