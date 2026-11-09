//! It reads at start, after every plan and when the window comes back to
//! the front, through a [`reader`](crate::reader).

#[cfg(test)]
mod tests;

use std::sync::Arc;

use parking_lot::Mutex;
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
    pub failed: Vec<FailedRead>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailedRead {
    pub probe: Probe,
    pub reason: String,
}

#[must_use = "the handle is the only way to the service"]
pub fn spawn<P>(runtime: &Handle, wake: Wake, probe: P) -> SnapshotHandle
where
    P: Fn() -> (Snapshot, Vec<ProbeIssue>) + Send + Sync + 'static,
{
    let logged = Mutex::new(Vec::new());

    reader::spawn(runtime, wake, "the system's state", move || {
        let (snapshot, issues) = probe();
        let probed = probed(snapshot, &issues);

        warn_on_change(&logged, &probed.failed);
        probed
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
            .filter_map(|issue| match &issue.error {
                ProbeError::Failed(reason) => Some(FailedRead {
                    probe: issue.probe,
                    reason: reason.clone(),
                }),
                ProbeError::Unavailable => None,
            })
            .collect(),
    }
}

/// Each read after a plan repeats a lasting failure, so the log names a
/// failure only when the set changes. Returns whether it logged.
fn warn_on_change(logged: &Mutex<Vec<FailedRead>>, failed: &[FailedRead]) -> bool {
    let mut last = logged.lock();

    if last.as_slice() == failed {
        return false;
    }

    for read in failed {
        tracing::warn!("could not read {}: {}", read.probe.as_str(), read.reason);
    }

    *last = failed.to_vec();
    true
}
