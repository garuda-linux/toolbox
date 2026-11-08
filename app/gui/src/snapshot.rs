//! It reads at start, after every plan and when the window comes back to
//! the front, through a [`reader`](crate::reader).

#[cfg(test)]
mod tests;

use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_core::Snapshot;
use toolbox_gui_kit::Loadable;
use toolbox_system::{Probe, ProbeError, ProbeIssue};

use crate::reader::{self, ReaderHandle};
use crate::waking::Wake;

pub type Reading = Loadable<Probed>;

pub type SnapshotHandle = ReaderHandle<Probed>;

#[derive(Debug)]
pub struct Probed {
    pub snapshot: Arc<Snapshot>,
    /// A part the system lacks is no failure and not listed.
    pub failed: Vec<Probe>,
}

#[must_use = "the handle is the only way to the service"]
pub fn spawn<P>(runtime: &Handle, wake: Wake, probe: P) -> SnapshotHandle
where
    P: Fn() -> (Snapshot, Vec<ProbeIssue>) + Send + Sync + 'static,
{
    reader::spawn(runtime, wake, "the system's state", move || {
        let (snapshot, issues) = probe();
        probed(snapshot, &issues)
    })
}

fn probed(snapshot: Snapshot, issues: &[ProbeIssue]) -> Probed {
    for issue in issues {
        tracing::debug!("probe {}: {}", issue.probe.as_str(), issue.error);
    }

    Probed {
        snapshot: Arc::new(snapshot),
        failed: issues
            .iter()
            .filter(|issue| matches!(issue.error, ProbeError::Failed(_)))
            .map(|issue| issue.probe)
            .collect(),
    }
}
