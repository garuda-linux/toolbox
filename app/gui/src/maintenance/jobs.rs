//! The Maintenance page's slow work: ranking the Chaotic-AUR mirrors and
//! copying settings from `/etc/skel`. Each runs on a blocking thread and
//! sends its result back for the next frame, which it wakes.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_catalog::SkelEntry;
use toolbox_plan::Mirrorlist;
use toolbox_system::mirrors::{MirrorRanking, RankProblem};
use toolbox_system::skel::shelf::{self, Outcome};

use crate::lane::Lane;

/// The self-test's Chaotic-AUR list. It never reaches a mirror.
const SAMPLE_MIRRORS: &str = "# Chaotic-AUR mirrors of the self-test, never ranked\n\
    Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n\
    Server = https://geo-mirror.chaotic.cx/$repo/$arch\n";

type Ranked = Result<Mirrorlist, RankProblem>;

/// What the page's jobs need from outside.
pub struct MaintenanceSetup {
    pub runtime: Handle,
    /// Ranks the Chaotic-AUR mirrors. A dry run's stand-in asks no mirror.
    pub mirrors: Arc<dyn MirrorRanking>,
    /// Where reset settings go.
    pub home: PathBuf,
    /// Where they come from, `/etc/skel` outside tests.
    pub skel_root: PathBuf,
}

#[derive(Debug)]
pub struct Jobs {
    setup: MaintenanceSetup,
    ranked: Lane<Ranked>,
    reset: Lane<Outcome>,
}

/// Answers a fixed list at once, for the dry run and its self-test:
/// `rate-mirrors` would run for minutes and ask every mirror.
#[derive(Clone, Copy, Debug)]
pub struct SampleMirrors;

impl MirrorRanking for SampleMirrors {
    fn rank(&self) -> Result<Mirrorlist, RankProblem> {
        Mirrorlist::new(SAMPLE_MIRRORS).map_err(|error| RankProblem::Invalid(error.to_string()))
    }
}

impl fmt::Debug for MaintenanceSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaintenanceSetup")
            .field("home", &self.home)
            .field("skel_root", &self.skel_root)
            .finish_non_exhaustive()
    }
}

impl Jobs {
    pub fn new(setup: MaintenanceSetup, ctx: egui::Context) -> Self {
        Self {
            ranked: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx.clone())),
            reset: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx)),
            setup,
        }
    }

    pub fn skel_root(&self) -> &Path {
        &self.setup.skel_root
    }

    /// A ranking that panicked counts as one that never started, so the
    /// page stops waiting for it.
    pub fn rank(&self) {
        let ranking = Arc::clone(&self.setup.mirrors);
        self.ranked.blocking(
            "rank",
            move || ranking.rank(),
            |ranked| ranked.unwrap_or_else(|error| Err(RankProblem::NotStarted(error.to_string()))),
        );
    }

    /// A reset that panicked counts as one that failed for every entry.
    pub fn reset(&self, entries: Vec<SkelEntry>) {
        let stopped = Outcome::stopped(&entries);
        let (skel_root, home) = (self.setup.skel_root.clone(), self.setup.home.clone());
        self.reset.blocking(
            "reset",
            move || shelf::reset(&entries, &skel_root, &home),
            |outcome| outcome.unwrap_or(stopped),
        );
    }

    pub fn ranked(&self) -> Option<Ranked> {
        self.ranked.landed().pop()
    }

    pub fn reset_outcome(&self) -> Option<Outcome> {
        self.reset.landed().pop()
    }
}
