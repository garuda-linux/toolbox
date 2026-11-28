//! The Maintenance page's slow work: ranking the Chaotic-AUR mirrors and
//! copying settings from `/etc/skel`. Each runs on a blocking thread and
//! sends its result back for the next frame, which it wakes.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::runtime::Handle;
use toolbox_catalog::SkelEntry;
use toolbox_plan::Mirrorlist;
use toolbox_system::mirrors::{MirrorRanking, RankProblem, StartProblem};
use toolbox_system::skel::shelf::{self, Outcome, SkelRow};

use crate::catalogs::Catalogs;
use crate::lane::Lane;

type Ranked = Result<Mirrorlist, RankProblem>;

/// What the page's jobs need from outside.
pub struct MaintenanceSetup {
    pub runtime: Handle,
    /// Ranks the Chaotic-AUR mirrors. The self-test's stand-in asks no
    /// mirror.
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
    offered: Lane<Vec<SkelRow>>,
    checked: Lane<bool>,
    /// Stops the ranking under way, which kills rate-mirrors.
    cancel: Arc<AtomicBool>,
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
            reset: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx.clone())),
            cancel: Arc::default(),
            offered: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx.clone())),
            checked: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx)),
            setup,
        }
    }

    #[cfg(test)]
    pub fn skel_root(&self) -> &Path {
        &self.setup.skel_root
    }

    /// A ranking that panicked counts as one that never started, so the
    /// page stops waiting for it.
    pub fn rank(&self) {
        let ranking = Arc::clone(&self.setup.mirrors);
        let cancel = Arc::clone(&self.cancel);
        cancel.store(false, Ordering::Relaxed);
        self.ranked.blocking(
            "rank",
            move || ranking.rank(&cancel),
            |ranked| ranked.unwrap_or(Err(RankProblem::NotStarted(StartProblem::Panicked))),
        );
    }

    /// Stops the ranking under way. Its result comes back as
    /// [`RankProblem::Cancelled`].
    pub(super) fn cancel_rank(&self) {
        self.cancel.store(true, Ordering::Relaxed);
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

    /// Reads what `/etc/skel` offers for the skel catalog, off the frame.
    pub fn offer(&self, catalogs: Arc<Catalogs>) {
        let skel_root = self.setup.skel_root.clone();
        self.offered.blocking(
            "offer",
            move || {
                catalogs
                    .skel
                    .as_ref()
                    .map(|catalog| shelf::offered(catalog, &skel_root))
                    .unwrap_or_default()
            },
            Result::unwrap_or_default,
        );
    }

    /// Asks the disk whether `program` is there, off the frame.
    pub fn check(&self, program: &'static str, exists: fn(&Path) -> bool) {
        self.checked.blocking(
            "check",
            move || exists(Path::new(program)),
            |found| found.unwrap_or(false),
        );
    }

    pub fn checked(&self) -> Option<bool> {
        self.checked.landed().pop()
    }

    pub fn offered(&self) -> Option<Vec<SkelRow>> {
        self.offered.landed().pop()
    }

    pub fn ranked(&self) -> Option<Ranked> {
        self.ranked.landed().pop()
    }

    pub(super) fn reset_outcome(&self) -> Option<Outcome> {
        self.reset.landed().pop()
    }
}
