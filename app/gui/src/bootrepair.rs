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
use toolbox_core::features::bootrepair::page::{Booted, Facts, Offer, Progress};
use toolbox_core::features::bootrepair::{Refusal, RepairFacts};
use toolbox_plan::Toggle;
use toolbox_system::{ProbeError, RepairSource};

use crate::command::{AppCommand, BootRepairCommand};
use crate::lane::Asked;
use crate::page_service::Handled;
use crate::refused::Refused;

type Answer = Result<RepairFacts, ProbeError>;

pub struct BootRepairSetup {
    pub runtime: Handle,
    pub source: Arc<dyn RepairSource>,
    /// `--boot-repair` was given: the page shows on an installed system.
    pub forced: bool,
}

/// The texts of Boot Repair name nothing.
pub type RepairRefusal = Refused<Refusal, ()>;

#[derive(Debug, Default)]
pub struct BootRepair {
    forced: bool,
    offer: Offer,
    facts: Facts,
    progress: Progress,
    refusal: Option<RepairRefusal>,
    /// `None` in tests that need no reads: nothing is read.
    reads: Option<Asked<Answer>>,
}

impl fmt::Debug for BootRepairSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootRepairSetup")
            .field("forced", &self.forced)
            .finish_non_exhaustive()
    }
}

impl BootRepair {
    pub const fn refusal(&self) -> Option<&RepairRefusal> {
        self.refusal.as_ref()
    }

    /// A refused repair stays on the page until dismissed or the next
    /// one starts.
    pub fn set_refusal(&mut self, refusal: Option<Refusal>) {
        self.refusal = refusal.map(|refusal| Refused::new(refusal, ()));
    }

    pub fn new(setup: Option<BootRepairSetup>, ctx: egui::Context) -> Self {
        let Some(setup) = setup else {
            return Self::default();
        };

        let forced = setup.forced;
        let mut state = Self {
            forced,
            reads: Some(Asked::new(
                setup.runtime,
                crate::waking::repaint(ctx),
                move |lane| {
                    let source = Arc::clone(&setup.source);
                    lane.blocking(
                        "facts",
                        move || source.facts(),
                        |answer| answer.unwrap_or(Err(ProbeError::Panicked)),
                    );
                },
            )),
            ..Self::default()
        };

        state.see_live(Booted::Installed);
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
    pub const fn see_live(&mut self, booted: Booted) {
        self.offer = Offer::of(booted, Toggle::from_on(self.forced));
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

    /// A repair runs at once in App's Console, as does its output.
    pub fn handle(&mut self, command: &AppCommand) -> Handled {
        let AppCommand::BootRepair(command) = command else {
            return Handled::default();
        };

        match command {
            BootRepairCommand::Refresh => self.refresh(),
            BootRepairCommand::Forget => self.forget(),
            BootRepairCommand::Repair {
                system: _,
                layout: _,
                passphrase: _,
                target: _,
            }
            | BootRepairCommand::ShowConsole => return Handled::default(),
        }

        Handled::taken()
    }

    pub fn take(&mut self) {
        let Some(answer) = self.reads.as_mut().and_then(Asked::take) else {
            return;
        };

        self.facts = match answer {
            Ok(facts) => Facts::Known(facts),
            Err(error) => {
                tracing::error!("the disks cannot be read: {error}");
                Facts::Failed
            }
        };
    }

    /// A hidden page reads nothing.
    fn read(&mut self) {
        if self.offer == Offer::Hidden {
            return;
        }

        let Some(reads) = &mut self.reads else {
            return;
        };

        if !reads.read() {
            return;
        }

        if !matches!(self.facts, Facts::Known(_)) {
            self.facts = Facts::Loading;
        }
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
