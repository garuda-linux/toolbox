//! Asks the root helper which accounts have a locked password. Only the
//! helper may read `/etc/shadow`, and it answers flags, never hashes.
//! Polkit lets an active session run `read` without a password.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use toolbox_exec::{Environment, Policy, capture};
use toolbox_plan::protocol::{ReadKind, decode};
use toolbox_plan::{ShadowLocks, UserName};

use crate::HelperReads;
use crate::pkexec::{self, HelperError, HelperRun, Refusal};

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
    Failed(HelperError),
}

pub struct HelperLocks(pub Arc<dyn HelperReads>);

impl LockSource for HelperLocks {
    fn locks(&self) -> Result<BTreeMap<UserName, bool>, LockProblem> {
        // The helper names the flags of people and of this account only,
        // which it learns from the socket.
        let policy = Policy::tail(MAX_ANSWER_BYTES)
            .identified()
            .capped()
            .within(LOCKS_DEADLINE);

        let answer = match pkexec::sorted(capture(
            &mut self.0.read_command(ReadKind::ShadowLocks),
            Environment::Inherited,
            &[],
            &policy,
            &Arc::default(),
        )) {
            HelperRun::Answered(answer) => answer,
            HelperRun::Refused(refusal) => return Err(LockProblem::Refused(refusal)),
            HelperRun::Failed(error) => return Err(LockProblem::Failed(error)),
            HelperRun::Cancelled => return Err(LockProblem::Failed(HelperError::Stopped)),
        };

        parse(&answer).ok_or(LockProblem::Failed(HelperError::Unreadable))
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
