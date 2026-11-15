//! The Users and groups page's own state: the lock flags the helper read
//! from `/etc/shadow`, and the password or account run the page started.
//!
//! The flags are asked once the page opens, after every plan and on
//! Refresh. Every read is a pkexec run, so the page never asks before it
//! is opened and at most one read is in flight.

mod edits;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_core::domains::AccountLockFlags;
use toolbox_core::features::users::run::{PageRun, RunKind};
use toolbox_plan::UserName;
use toolbox_services::locks::Locks;
use toolbox_system::pkexec::HelperError;
use toolbox_system::{LockProblem, LockSource};

use crate::lane::Asked;
use crate::refused::Refused;

pub type UsersRefusal = Refused<toolbox_core::features::users::Refusal>;

type Answer = Result<BTreeMap<UserName, bool>, LockProblem>;

pub struct UsersSetup {
    pub runtime: Handle,
    pub locks: Arc<dyn LockSource>,
}

#[derive(Debug, Default)]
pub struct Users {
    locks: Locks,
    run: PageRun,
    refusal: Option<UsersRefusal>,
    /// `None` in tests that need no reads: nothing is read.
    reads: Option<Asked<Answer>>,
}

impl fmt::Debug for UsersSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UsersSetup").finish_non_exhaustive()
    }
}

impl Users {
    pub fn new(setup: Option<UsersSetup>, ctx: egui::Context) -> Self {
        Self {
            reads: setup.map(|setup| {
                Asked::new(
                    setup.runtime.clone(),
                    crate::waking::repaint(ctx),
                    move |lane| {
                        let source = Arc::clone(&setup.locks);
                        lane.blocking(
                            "locks",
                            move || source.locks(),
                            |answer| {
                                answer.unwrap_or(Err(LockProblem::Failed(HelperError::Stopped)))
                            },
                        );
                    },
                )
            }),
            ..Self::default()
        }
    }

    pub const fn locks(&self) -> &Locks {
        &self.locks
    }

    pub const fn run(&self) -> &PageRun {
        &self.run
    }

    pub const fn refusal(&self) -> Option<&UsersRefusal> {
        self.refusal.as_ref()
    }

    /// For a run the page starts at once, which App sends.
    pub fn set_refusal(&mut self, refusal: Option<UsersRefusal>) {
        self.refusal = refusal;
    }

    /// The snapshot part the guards check, `None` while unknown.
    pub fn flags(&self) -> Option<AccountLockFlags> {
        self.locks.flags()
    }

    pub fn open(&mut self) {
        if self.locks == Locks::NotAsked {
            self.read();
        }
    }

    /// For Refresh, because a lock set outside the app shows in no probe.
    pub fn refresh(&mut self) {
        self.read();
    }

    pub fn started(&mut self, kind: RunKind, user: UserName) {
        self.run = PageRun::Sent(kind, user);
    }

    pub fn plan_started(&mut self) {
        self.run = std::mem::take(&mut self.run).started();
    }

    pub fn plan_refused(&mut self) {
        self.run = std::mem::take(&mut self.run).refused();
    }

    pub fn dismiss_run(&mut self) {
        self.run = PageRun::None;
    }

    /// A plan ended: it may have locked an account or set a password.
    pub fn plan_finished(&mut self, succeeded: bool) {
        self.run = std::mem::take(&mut self.run).finished(succeeded);
        if self.locks != Locks::NotAsked {
            self.read();
        }
    }

    /// Takes the answers since the last frame. Returns whether the lock
    /// flags changed, so the snapshot takes them.
    pub fn take(&mut self) -> bool {
        let Some(answer) = self.reads.as_mut().and_then(Asked::take) else {
            return false;
        };

        if let Err(problem) = &answer {
            tracing::info!("the lock state of accounts is unknown: {problem:?}");
        }

        let before = self.flags();
        self.locks = Locks::of_answer(answer);

        before != self.flags()
    }

    fn read(&mut self) {
        let Some(reads) = &mut self.reads else {
            return;
        };

        if !reads.read() {
            return;
        }

        if !matches!(self.locks, Locks::Known(_)) {
            self.locks = Locks::Loading;
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Users {
    pub fn showing(locks: Locks, run: PageRun) -> Self {
        Self {
            locks,
            run,
            ..Self::default()
        }
    }
}
