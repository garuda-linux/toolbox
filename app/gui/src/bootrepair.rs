//! The Boot repair page's own state: whether the page is offered, what
//! the read of the disks answered, and how the repair it started went.
//!
//! The page exists only on the live ISO, or with `--boot-repair` for
//! testing. A hidden page reads nothing. Core's [`page`] decides every
//! state and every pick.

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_core::features::bootrepair::RepairFacts;
use toolbox_core::features::bootrepair::page::{Facts, Offer, Progress};
use toolbox_system::{ProbeError, RepairSource};

use crate::lane::{Lane, OneRead};

type Answer = Result<RepairFacts, ProbeError>;

pub struct BootRepairSetup {
    pub runtime: Handle,
    pub source: Arc<dyn RepairSource>,
    /// `--boot-repair` was given: the page shows on an installed system.
    pub forced: bool,
}

#[derive(Debug, Default)]
pub struct BootRepair {
    forced: bool,
    offer: Offer,
    facts: Facts,
    progress: Progress,
    reads: OneRead,
    /// `None` in tests that need no reads: nothing is read.
    jobs: Option<Jobs>,
}

struct Jobs {
    source: Arc<dyn RepairSource>,
    lane: Lane<Answer>,
}

impl fmt::Debug for BootRepairSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootRepairSetup")
            .field("forced", &self.forced)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Jobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Jobs").finish_non_exhaustive()
    }
}

impl BootRepair {
    pub fn new(setup: Option<BootRepairSetup>, ctx: egui::Context) -> Self {
        let Some(setup) = setup else {
            return Self::default();
        };

        let forced = setup.forced;
        let mut state = Self {
            forced,
            jobs: Some(Jobs {
                source: setup.source,
                lane: Lane::new(setup.runtime, crate::waking::repaint(ctx)),
            }),
            ..Self::default()
        };

        state.see_live(false);
        state
    }

    pub const fn offer(&self) -> Offer {
        self.offer
    }

    pub const fn facts(&self) -> &Facts {
        &self.facts
    }

    pub const fn progress(&self) -> Progress {
        self.progress
    }

    pub const fn offered(&self) -> bool {
        !matches!(self.offer, Offer::Hidden)
    }

    /// It may come after the start, so the offer is worked out again on every look.
    pub const fn see_live(&mut self, live: bool) {
        self.offer = Offer::of(live, self.forced);
    }

    pub fn open(&mut self) {
        if self.facts == Facts::NotAsked {
            self.read();
        }
    }

    pub fn refresh(&mut self) {
        self.read();
    }

    pub const fn started(&mut self) {
        self.progress = Progress::Waiting;
    }

    pub const fn plan_started(&mut self) {
        self.progress = self.progress.started();
    }

    pub const fn plan_refused(&mut self) {
        self.progress = self.progress.refused();
    }

    /// A plan ended. The repair of this page changes what the disks hold
    /// and may have left something unlocked after a failure, so the page
    /// reads them again.
    pub fn plan_finished(&mut self, succeeded: bool) {
        if self.progress == Progress::Running {
            self.progress = self.progress.finished(succeeded);
            self.read();
        }
    }

    pub const fn forget(&mut self) {
        self.progress = self.progress.forgotten();
    }

    pub fn take(&mut self) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        let Some(answer) = jobs.lane.landed().pop() else {
            return;
        };

        self.facts = match answer {
            Ok(facts) => Facts::Known(facts),
            Err(error) => {
                tracing::error!("the disks cannot be read: {error}");
                Facts::Failed
            }
        };

        if self.reads.landed() {
            self.read();
        }
    }

    /// A hidden page reads nothing.
    fn read(&mut self) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        if self.offer == Offer::Hidden {
            return;
        }

        if !self.reads.start() {
            return;
        }

        if !matches!(self.facts, Facts::Known(_)) {
            self.facts = Facts::Loading;
        }

        let source = Arc::clone(&jobs.source);
        jobs.lane.blocking(
            "facts",
            move || source.facts(),
            |answer| answer.unwrap_or_else(|stopped| Err(ProbeError::Failed(stopped.to_string()))),
        );
    }
}

#[cfg(any(test, feature = "testing"))]
impl BootRepair {
    pub fn showing(offer: Offer, facts: Facts, progress: Progress) -> Self {
        Self {
            forced: offer == Offer::Test,
            offer,
            facts,
            progress,
            ..Self::default()
        }
    }
}
