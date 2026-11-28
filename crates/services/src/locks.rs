//! What the Users page knows of the locked passwords: not asked yet,
//! asking, or the helper's answer. `toolbox-system`'s `LockSource` reads
//! them.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use toolbox_core::features::ReadState;
use toolbox_plan::{Toggle, UserName};
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

    pub fn of(&self, user: &UserName) -> Option<Toggle> {
        match self {
            Self::Known(flags) => flags.get(user).copied().map(Toggle::from_on),
            Self::NotAsked | Self::Loading | Self::Refused(_) | Self::Unknown => None,
        }
    }

    /// How the guards that wait for the flags see this read.
    pub const fn read_state(&self) -> ReadState {
        match self {
            Self::Refused(_) | Self::Unknown => ReadState::Failed,
            Self::NotAsked | Self::Loading | Self::Known(_) => ReadState::Waiting,
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
}
