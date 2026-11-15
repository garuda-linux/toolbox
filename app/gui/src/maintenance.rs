//! What the Maintenance page keeps between frames: why the last tool did
//! not start or the mirrors were not ranked, the ranking under way, the
//! install a missing tool waits for and the settings reset.
//!
//! The ranking and the reset block, so they run on tokio's blocking
//! threads and the frame takes their results.

mod jobs;
mod mirrorlist;
#[cfg(any(test, feature = "selftest", feature = "testing"))]
mod sample;
#[cfg(test)]
mod tests;

use std::path::Path;
use std::sync::Arc;

use jobs::Jobs;
pub use jobs::MaintenanceSetup;
use mirrorlist::Install;
#[cfg(any(test, feature = "selftest", feature = "testing"))]
pub use sample::SampleMirrors;
use toolbox_catalog::SkelCatalog;
use toolbox_core::features::maintenance::opening::{Next, Opening};
use toolbox_core::features::maintenance::page::{
    self, LaunchCause, PageTool, RATE_MIRRORS_PROGRAM, ToolRow,
};
use toolbox_core::features::maintenance::{RATE_MIRRORS_PACKAGE, Tool};
use toolbox_plan::{Plan, RootAction, Toggle};
use toolbox_services::shelf::Shelf;
use toolbox_system::mirrors::RankProblem;
use toolbox_system::skel::shelf;

use crate::catalogs::Catalogs;
use crate::command::{AppCommand, MaintenanceCommand};
use crate::launch::Launcher;
use crate::page_service::Handled;

/// Why the last tool did not start or the mirrors were not ranked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaintenanceProblem {
    /// The tool `tool` could not start. `reason` is the system's own text.
    Launch {
        tool: &'static str,
        cause: LaunchCause,
        reason: String,
    },
    Mirrors(RankProblem),
    /// A run of the page was not started, as another plan runs.
    Busy,
}

#[derive(Debug, Default)]
pub struct Maintenance {
    problem: Option<MaintenanceProblem>,
    /// The Chaotic-AUR mirrors are being ranked, which takes minutes.
    ranking: bool,
    /// The install of the last ranked list, until it started.
    mirrorlist: Install,
    /// The settings reset: what is on offer, what is chosen, what happened.
    shelf: Shelf,
    opening: Opening,
    /// `None` in tests that need no jobs: nothing is ranked or reset.
    jobs: Option<Jobs>,
    /// Starts the tool that waited for its install.
    launcher: Option<Arc<dyn Launcher>>,
    /// Shared with App, which loads them once at the start. The reset
    /// copies what the skel catalog names.
    catalogs: Arc<Catalogs>,
}

impl Maintenance {
    pub fn new(setup: Option<MaintenanceSetup>, ctx: egui::Context) -> Self {
        Self {
            jobs: setup.map(|setup| Jobs::new(setup, ctx)),
            ..Self::default()
        }
    }

    pub fn set_launcher(&mut self, launcher: Option<Arc<dyn Launcher>>) {
        self.launcher = launcher;
    }

    /// Also offers what `/etc/skel` holds of their skel catalog.
    pub fn set_catalogs(&mut self, catalogs: Arc<Catalogs>) {
        self.catalogs = catalogs;
        self.restock();
    }

    /// Offers what `/etc/skel` holds now, which an apply may have changed.
    pub fn restock(&mut self) {
        let catalogs = Arc::clone(&self.catalogs);
        self.stock(catalogs.skel.as_ref());
    }

    pub fn launcher(&self) -> Option<Arc<dyn Launcher>> {
        self.launcher.clone()
    }

    /// Offers what `/etc/skel` holds now of `catalog`.
    pub fn stock(&mut self, catalog: Option<&SkelCatalog>) {
        let rows = catalog
            .zip(self.jobs.as_ref())
            .map(|(catalog, jobs)| shelf::offered(catalog, jobs.skel_root()))
            .unwrap_or_default();

        self.shelf.stock(rows);
    }

    /// Opens `tool`, or returns the install of its package. `tools` is
    /// what the last check found.
    pub fn open_tool(
        &mut self,
        tool: PageTool,
        tools: &[ToolRow],
        launcher: Option<&dyn Launcher>,
    ) -> Option<Plan> {
        let installed = page::installed(tools, tool);
        let tool = match tool {
            PageTool::ChaoticMirrors if installed => {
                self.rank();
                return None;
            }
            PageTool::ChaoticMirrors => {
                self.opening = Opening::Sent(Next::RankMirrors);
                return Some(page::install(&RATE_MIRRORS_PACKAGE));
            }
            PageTool::Program(tool) => tool,
        };

        if installed {
            self.launch(tool, launcher);
            return None;
        }

        self.opening = Opening::Sent(Next::Tool(tool));
        Some(page::install(&tool.package()))
    }

    pub const fn problem(&self) -> Option<&MaintenanceProblem> {
        self.problem.as_ref()
    }

    pub const fn ranking(&self) -> bool {
        self.ranking
    }

    pub const fn shelf(&self) -> &Shelf {
        &self.shelf
    }

    /// Chooses the settings `entry` for the reset, or drops it.
    pub fn choose(&mut self, entry: &str, wish: Toggle) {
        self.shelf.choose(entry, wish);
    }

    pub fn dismiss_problem(&mut self) {
        self.problem = None;
    }

    /// App did not send a run of the page, as another plan runs.
    pub fn refuse_busy(&mut self) {
        self.problem = Some(MaintenanceProblem::Busy);
    }

    /// The runs, the queue and Refresh need what App holds: the Console
    /// and garuda-update's program.
    pub fn handle(&mut self, command: &AppCommand) -> Handled {
        let AppCommand::Maintenance(command) = command else {
            return Handled::default();
        };

        match command {
            MaintenanceCommand::DismissProblem => self.dismiss_problem(),
            MaintenanceCommand::Choose { entry, on } => self.choose(entry, Toggle::from_on(*on)),
            MaintenanceCommand::Reset => {
                let catalogs = Arc::clone(&self.catalogs);
                self.reset(catalogs.skel.as_ref());
            }
            MaintenanceCommand::RunOperation(_)
            | MaintenanceCommand::RunRepair(_)
            | MaintenanceCommand::Queue {
                operation: _,
                on: _,
            }
            | MaintenanceCommand::OpenTool(_)
            | MaintenanceCommand::InstallMissing
            | MaintenanceCommand::Reveal(_)
            | MaintenanceCommand::Refresh => return Handled::default(),
        }

        Handled::taken()
    }

    /// The run of a plan began, the one this page sent if it waits.
    pub const fn plan_started(&mut self) {
        self.opening = self.opening.started();
    }

    /// Another plan was running, so the install never started.
    pub const fn plan_refused(&mut self) {
        self.opening = self.opening.refused();
    }

    /// The install of the ranked list began.
    pub fn mirrorlist_started(&mut self) {
        self.mirrorlist = std::mem::take(&mut self.mirrorlist).started();
    }

    /// Another plan was running, so the list waits for its end.
    pub fn mirrorlist_refused(&mut self) {
        self.mirrorlist = std::mem::take(&mut self.mirrorlist).refused();
    }

    /// Starts what waited for the install that just ended, when `exists`
    /// finds its program now: the last check may have run before the end.
    /// A ranked list that waited goes out with the next [`Self::take`].
    pub fn plan_finished(
        &mut self,
        succeeded: bool,
        exists: impl Fn(&Path) -> bool,
        launcher: Option<&dyn Launcher>,
    ) {
        self.mirrorlist = std::mem::take(&mut self.mirrorlist).finished();
        let (next, left) = self.opening.finished(succeeded);
        self.opening = left;
        match next {
            Some(Next::Tool(tool)) if exists(Path::new(tool.program())) => {
                self.launch(tool, launcher);
            }
            Some(Next::Tool(tool)) => self.absent(PageTool::Program(tool), tool.program()),
            Some(Next::RankMirrors) if exists(Path::new(RATE_MIRRORS_PROGRAM)) => self.rank(),
            Some(Next::RankMirrors) => self.absent(PageTool::ChaoticMirrors, RATE_MIRRORS_PROGRAM),
            None => {}
        }
    }

    /// Copies the chosen settings from `/etc/skel` on a blocking thread.
    pub fn reset(&mut self, catalog: Option<&SkelCatalog>) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        if let Some(entries) = catalog.and_then(|catalog| self.shelf.begin_reset(catalog)) {
            jobs.reset(entries);
        }
    }

    /// The results that came in since the last frame. A ranked list comes
    /// back as the plan that installs it, and so does one that waited for
    /// the end of another plan.
    pub fn take(&mut self) -> Option<Plan> {
        let jobs = self.jobs.as_ref()?;
        if let Some(outcome) = jobs.reset_outcome() {
            self.shelf.finish_reset(outcome);
        }

        if let Some(plan) = self.mirrorlist.due() {
            return Some(plan);
        }

        let ranked = jobs.ranked()?;
        self.ranking = false;
        match ranked {
            Ok(mirrorlist) => {
                let plan = Plan::root(RootAction::InstallMirrorlist { mirrorlist });
                Some(self.mirrorlist.landed(plan))
            }
            Err(problem) => {
                tracing::error!("ranking the Chaotic-AUR mirrors failed: {problem}");
                self.problem = Some(MaintenanceProblem::Mirrors(problem));
                None
            }
        }
    }

    /// A second ranking while one runs would test every mirror twice.
    fn rank(&mut self) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        if self.ranking {
            return;
        }

        self.ranking = true;
        self.problem = None;
        jobs.rank();
    }

    /// Without this the "Install and open…" click would end in silence.
    fn absent(&mut self, tool: PageTool, program: &str) {
        tracing::error!("{program} is missing after its install ended well");
        self.problem = Some(MaintenanceProblem::Launch {
            tool: tool.key(),
            cause: LaunchCause::AbsentAfterInstall,
            reason: program.to_owned(),
        });
    }

    /// A failure is logged here and kept for the page.
    fn launch(&mut self, tool: Tool, launcher: Option<&dyn Launcher>) {
        let Some(launcher) = launcher else {
            return;
        };

        let program = Path::new(tool.program());
        self.problem = launcher.start(program, tool.args()).err().map(|error| {
            tracing::error!("cannot start {}: {error}", program.display());
            MaintenanceProblem::Launch {
                tool: tool.id(),
                cause: LaunchCause::of(error.kind()),
                reason: error.to_string(),
            }
        });
    }
}

#[cfg(any(test, feature = "testing"))]
impl Maintenance {
    /// What a page test shows: `problem`, a ranking under way or not, the
    /// `rows` on offer for the reset and the last reset's `outcome`.
    pub fn showing(
        problem: Option<MaintenanceProblem>,
        ranking: bool,
        rows: Vec<shelf::SkelRow>,
        outcome: Option<shelf::Outcome>,
    ) -> Self {
        let mut shelf = Shelf::default();
        shelf.stock(rows);
        if let Some(outcome) = outcome {
            shelf.finish_reset(outcome);
        }

        Self {
            problem,
            ranking,
            shelf,
            ..Self::default()
        }
    }
}
