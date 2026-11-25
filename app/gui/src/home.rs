//! Home's readers and its note: the status checks, the facts, the news,
//! the programs found, and why the last restart or installer failed.

mod merge_reads;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::Receiver;
use toolbox_core::features::maintenance::page::{RepairRow, ToolRow};
use toolbox_core::features::maintenance::{Program, queue_upgrade};
use toolbox_core::features::merges;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui_kit::Loadable;
use toolbox_news::News;
use toolbox_plan::MergeCopy;
use toolbox_system::{CallError, Facts, Status};

pub use self::merge_reads::{MergeReads, Merges};
use crate::command::{AppCommand, HomeCommand};
use crate::page::PageId;
use crate::page_service::Handled;
use crate::pages::home::{HomeFacts, HomeProblem};
use crate::power::Power;
use crate::programs::Programs;
use crate::reader::{ReaderHandle, Watched};
use crate::refused::queue;
use crate::waking::{self, Wake, Waking};

/// The readers Home shows. `None` reads nothing, as in tests.
#[derive(Debug, Default)]
pub struct HomeReaders {
    pub status: Option<ReaderHandle<Status>>,
    pub facts: Option<ReaderHandle<Facts>>,
    pub news: Option<ReaderHandle<News>>,
    pub programs: Option<ReaderHandle<Programs>>,
}

#[derive(Debug)]
pub struct Home {
    status: Watched<Status>,
    facts: Watched<Facts>,
    news: Watched<News>,
    programs: Watched<Programs>,
    /// `None` leaves Restart without effect.
    power: Option<Arc<dyn Power>>,
    /// Refusals of a restart, sent from logind's task.
    restart_refused: (Waking<CallError>, Receiver<CallError>),
    problem: Option<HomeProblem>,
    merges: Merges,
}

impl Default for Home {
    fn default() -> Self {
        Self::new(HomeReaders::default(), None, Wake::none())
    }
}

impl Home {
    /// `wake` brings the window back for a refused restart.
    pub fn new(readers: HomeReaders, power: Option<Arc<dyn Power>>, wake: Wake) -> Self {
        Self {
            status: Watched::new(readers.status),
            facts: Watched::new(readers.facts),
            news: Watched::new(readers.news),
            programs: Watched::new(readers.programs),
            power,
            restart_refused: waking::channel(wake),
            problem: None,
            merges: Merges::default(),
        }
    }

    /// What Home shows, with what the window keeps across starts.
    pub fn shown<'a>(&'a self, recent: &'a [PageId], setup_finished: bool) -> HomeFacts<'a> {
        HomeFacts {
            status: self.status.fed(),
            status_revision: self.status.revision(),
            facts: self.facts.fed(),
            news: self.news.fed(),
            recent,
            setup_finished,
            programs: self.programs.latest(),
            programs_failed: self.programs.reading().error().is_some(),
            problem: self.problem.as_ref(),
        }
    }

    #[cfg(feature = "testing")]
    pub const fn awaits(&self) -> bool {
        self.status.awaits() || self.facts.awaits() || self.news.awaits() || self.programs.awaits()
    }

    pub fn set_merge_reads(&mut self, reads: MergeReads) {
        self.merges.set_reads(reads);
    }

    pub const fn merges(&self) -> &Merges {
        &self.merges
    }

    /// For tests, which read nothing.
    pub const fn merges_mut(&mut self) -> &mut Merges {
        &mut self.merges
    }

    pub fn set_problem(&mut self, problem: Option<HomeProblem>) {
        self.problem = problem;
    }

    /// Whether the last check found garuda-update's program, for a system
    /// whose installed packages are not known.
    pub fn garuda_update(&self) -> bool {
        self.programs
            .latest()
            .is_some_and(|programs| programs.garuda_update)
    }

    /// The tools Maintenance opens, none before the first check.
    pub fn tools(&self) -> &[ToolRow] {
        self.programs
            .latest()
            .map_or(&[][..], |programs| programs.tools.as_slice())
    }

    /// The repairs of Maintenance with what each needs, none before the
    /// first check.
    pub fn repairs(&self) -> &[RepairRow] {
        self.programs
            .latest()
            .map_or(&[][..], |programs| programs.repairs.as_slice())
    }

    /// The copies the last status check found that the toolbox may open.
    fn merge_copies(&self) -> Vec<MergeCopy> {
        self.status
            .fed()
            .and_then(Loadable::get)
            .and_then(|status| status.merges.as_ref().ok())
            .into_iter()
            .flatten()
            .filter_map(|path| path.to_str())
            .filter_map(|path| MergeCopy::new(path).ok())
            .collect()
    }

    pub fn status_every(&self, period: Option<Duration>) {
        self.status.every(period);
    }

    /// The counts Home shows change with the packages.
    pub fn refresh_packages(&self) {
        self.status.refresh();
        self.facts.refresh();
    }

    pub fn refresh_programs(&self) {
        self.programs.refresh();
    }

    /// The note goes, as a new attempt may fail for another reason.
    pub fn restart(&mut self) {
        self.problem = None;
        if let Some(power) = &self.power {
            power.restart(self.restart_refused.0.clone());
        }
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Home(command) = command else {
            return Handled::default();
        };

        match command {
            HomeCommand::QueueUpgrade(on) => {
                let program = Program::of_found(self.garuda_update());
                queue(changes, |changes| {
                    queue_upgrade(Toggle::from_on(*on), changes, snapshot, program);
                })
            }
            HomeCommand::DismissProblem => {
                self.problem = None;
                Handled::taken()
            }
            HomeCommand::ReadMerges => {
                self.merges.read(self.merge_copies());
                Handled::taken()
            }
            HomeCommand::CloseMerges => {
                self.merges.stop();
                Handled::taken()
            }
            HomeCommand::Merge { copy, choice } => queue(changes, |changes| match choice {
                Some(choice) => merges::queue(changes, copy.clone(), choice.clone(), snapshot),
                None => merges::take_back(changes, copy),
            }),
            HomeCommand::Refresh => {
                self.status.refresh();
                self.facts.refresh();
                self.news.refresh();
                self.programs.refresh();
                Handled::answers(vec![AppCommand::ReadSystem])
            }
            // These need the Console, the launcher or the saved window state.
            HomeCommand::RunNow(_)
            | HomeCommand::Restart
            | HomeCommand::StartInstaller
            | HomeCommand::SetupFinished => Handled::default(),
        }
    }

    /// A failed read shows on the page, which offers to read again.
    pub fn take(&mut self) {
        self.status.take();
        self.facts.take();
        self.news.take();
        self.programs.take();
        self.merges.take();
        if let Some(error) = self.restart_refused.1.try_iter().last() {
            self.problem = Some(HomeProblem::Restart(error));
        }
    }

    /// A plan may have changed the packages and programs they count.
    pub fn plan_finished(&self) {
        self.status.refresh();
        self.facts.refresh();
        self.programs.refresh();
    }
}
