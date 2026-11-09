//! Asks the root helper which accounts have a locked password. Only the
//! helper may read `/etc/shadow`, and it answers flags, never hashes.
//! Polkit lets an active session run `read` without a password.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use toolbox_exec::{CaptureError, Environment, Policy, capture};
use toolbox_plan::protocol::{ReadKind, decode};
use toolbox_plan::{ShadowLocks, UserName};
use toolbox_state::accounts::AccountLockFlags;

use crate::HelperReads;
use crate::pkexec::{self, Refusal};

/// One flag per account. Far more than any system has, far less than a
/// helper gone wrong could print.
const MAX_ANSWER_BYTES: usize = 1 << 20;

/// Polkit lets an active session read without a dialog, so the helper
/// answers within a moment. One that hangs must not hold the Users page.
const LOCKS_DEADLINE: Duration = Duration::from_secs(60);

/// Answers which accounts are locked. Blocks, so call it off the GUI
/// thread.
pub trait LockSource: Send + Sync {
    /// # Errors
    ///
    /// Why the flags are not known.
    fn locks(&self) -> Result<BTreeMap<UserName, bool>, LockProblem>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockProblem {
    Refused(Refusal),
    Failed(String),
}

pub struct HelperLocks(pub Arc<dyn HelperReads>);

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

impl LockSource for HelperLocks {
    fn locks(&self) -> Result<BTreeMap<UserName, bool>, LockProblem> {
        // The helper names the flags of people and of this account only,
        // which it learns from the socket.
        let policy = Policy::tail(MAX_ANSWER_BYTES)
            .identified()
            .capped()
            .within(LOCKS_DEADLINE);

        let captured = capture(
            &mut self.0.read_command(ReadKind::ShadowLocks),
            Environment::Inherited,
            &[],
            &policy,
            &Arc::default(),
        )
        .map_err(|error| LockProblem::Failed(unanswered(&error)))?;

        if let Some(refusal) = pkexec::refusal(captured.code, &captured.errors) {
            return Err(LockProblem::Refused(refusal));
        }

        if captured.code != Some(0_i32) {
            return Err(LockProblem::Failed(pkexec::ended(
                captured.code,
                &captured.errors,
            )));
        }

        parse(&captured.output).ok_or_else(|| {
            LockProblem::Failed("the helper's answer is not a list of locks".to_owned())
        })
    }
}

fn unanswered(error: &CaptureError) -> String {
    match error {
        CaptureError::Missing | CaptureError::Failed(_) => {
            format!("cannot start the helper: {error}")
        }
        CaptureError::TimedOut => format!(
            "the helper gave no answer within {} seconds",
            LOCKS_DEADLINE.as_secs()
        ),
        CaptureError::TooLarge => "the helper's answer is too long".to_owned(),
        CaptureError::Cancelled => "the read was cancelled".to_owned(),
    }
}

/// The helper's one answer line as a map. Names are validated while
/// decoding, so an answer with anything else is refused whole.
fn parse(answer: &str) -> Option<BTreeMap<UserName, bool>> {
    let locks: ShadowLocks = decode(answer.trim_end()).ok()?;

    Some(
        locks
            .locks
            .into_iter()
            .map(|lock| (lock.user, lock.locked))
            .collect(),
    )
}
