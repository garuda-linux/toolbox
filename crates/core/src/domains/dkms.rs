//! DKMS modules to build for a kernel. A build is work, not a state the
//! snapshot holds (`dkms status` is a slow program the Kernels page asks
//! on its own), so a queued build leaves the queue after a successful
//! apply like the Maintenance operations.

#[cfg(test)]
mod tests;

use toolbox_plan::{DkmsBuild, RootAction};

use super::each;
use crate::domain::{Domain, Queued, Stage, Staged, Summary};
use crate::{Snapshot, SummaryCode};

pub struct DkmsBuilds;

impl Domain for DkmsBuilds {
    type Item = DkmsBuild;
    type State = ();
    type Wish = Queued;

    const ID: &'static str = "dkms-modules";
    const ONE_OFF: bool = true;
    const STAGE: Stage = Stage::DKMS_BUILDS;
    const CODES: &'static [SummaryCode] = &[SummaryCode::DkmsBuild];

    fn state(_snapshot: &Snapshot) -> Option<&()> {
        Some(&())
    }

    fn is_satisfied(_item: &DkmsBuild, _wish: &Queued, (): &()) -> bool {
        false
    }

    /// dkms takes one module and one kernel per call.
    fn plan(changes: &[(&DkmsBuild, &Queued)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |build, _| {
            RootAction::InstallDkmsModule {
                build: build.clone(),
            }
        })
    }

    fn key(item: &DkmsBuild) -> String {
        format!("{}/{} {}", item.module, item.version, item.kernel)
    }

    fn summary(item: &DkmsBuild, _wish: &Queued) -> Summary {
        Summary::pair(
            SummaryCode::DkmsBuild,
            format!("{}/{}", item.module, item.version),
            item.kernel.as_str(),
        )
    }
}
