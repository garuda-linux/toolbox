//! Finished installs and the disk check that decides what they start.

use std::path::Path;
use std::sync::Arc;

use toolbox_core::features::maintenance::opening::Next;
use toolbox_core::features::maintenance::page::{PageTool, RATE_MIRRORS_PROGRAM};

use super::jobs::Jobs;
use super::{Checking, Maintenance};
use crate::launch::Launcher;

impl Maintenance {
    /// Starts what waited for the install that just ended, when `exists`
    /// finds its program now: the last check may have run before the end.
    /// A ranked list that waited goes out with the next [`Self::take`].
    pub fn plan_finished(
        &mut self,
        succeeded: bool,
        exists: fn(&Path) -> bool,
        launcher: Option<Arc<dyn Launcher>>,
    ) {
        self.mirrorlist = std::mem::take(&mut self.mirrorlist).finished();
        let (next, left) = self.opening.finished(succeeded);
        self.opening = left;
        let Some(next) = next else {
            return;
        };

        // The disk is asked off the frame. Without jobs it is asked here.
        if let Some(jobs) = &self.jobs {
            jobs.check(program_of(next), exists);
            self.checking = Some(Checking { next, launcher });
        } else {
            let found = exists(Path::new(program_of(next)));
            self.decide(next, found, launcher.as_deref());
        }
    }

    /// Acts on the disk check that landed, if one did.
    pub(super) fn land_check(&mut self) {
        let Some(found) = self.jobs.as_ref().and_then(Jobs::checked) else {
            return;
        };

        let Some(Checking { next, launcher }) = self.checking.take() else {
            return;
        };

        self.decide(next, found, launcher.as_deref());
    }

    /// Starts what waited for the install, or says it is missing.
    pub(super) fn decide(&mut self, next: Next, found: bool, launcher: Option<&dyn Launcher>) {
        match next {
            Next::Tool(tool) if found => self.launch(tool, launcher),
            Next::Tool(tool) => self.absent(PageTool::Program(tool), tool.found_by()),
            Next::RankMirrors if found => self.rank(),
            Next::RankMirrors => self.absent(PageTool::ChaoticMirrors, RATE_MIRRORS_PROGRAM),
        }
    }
}

/// The program a finished install's check looks for.
const fn program_of(next: Next) -> &'static str {
    match next {
        Next::Tool(tool) => tool.found_by(),
        Next::RankMirrors => RATE_MIRRORS_PROGRAM,
    }
}
