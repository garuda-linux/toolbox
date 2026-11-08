//! The Home page's system status: pending updates, files waiting for a
//! merge, the last update and garuda-health's findings.
//!
//! These probes are slow (checkupdates syncs a database copy, paru asks
//! the AUR), so they have their own [`StatusSources`] and
//! [`probe_status`] apart from [`crate::probe_snapshot`]. The change queue
//! never waits for them.

mod health;
mod pacdiff;
mod pacman_log;
#[cfg(test)]
mod tests;
mod timestamp;
mod updates;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

pub use health::{GarudaHealth, HealthIssue, HealthLevel, HealthSource};
pub use pacdiff::{MergeSource, Pacdiff};
pub use pacman_log::{PacmanLog, RecentLogSource, UpdateLogSource};
pub use updates::{AurUpdateSource, Checkupdates, Paru, PendingUpdate, UpdateSource};

use crate::error::ProbeError;
use crate::host::{HostSource, SystemHost};

/// The ports the status is probed from. Shared pointers, so a clone can
/// move into `spawn_blocking`.
#[derive(Clone)]
pub struct StatusSources {
    pub updates: Arc<dyn UpdateSource>,
    /// AUR updates, asked as the user.
    pub aur_updates: Arc<dyn AurUpdateSource>,
    pub merges: Arc<dyn MergeSource>,
    pub update_log: Arc<dyn UpdateLogSource>,
    pub health: Arc<dyn HealthSource>,
    /// Whether this is the live ISO, where health is skipped.
    pub host: Arc<dyn HostSource>,
}

/// The status as one snapshot part. Each field keeps its own result, so
/// the page can say "not available" for one card and show the others.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub updates: Result<Vec<PendingUpdate>, ProbeError>,
    pub aur_updates: Result<Vec<PendingUpdate>, ProbeError>,
    /// `.pacnew`, `.pacsave` and `.pacorig` files.
    pub merges: Result<Vec<PathBuf>, ProbeError>,
    /// The last completed transaction, `None` when the log has none.
    pub last_update: Result<Option<SystemTime>, ProbeError>,
    /// garuda-health's issues. Unavailable on the live ISO.
    pub health: Result<Vec<HealthIssue>, ProbeError>,
}

impl StatusSources {
    #[must_use = "the sources do nothing until probed"]
    pub fn system() -> Self {
        Self {
            updates: Arc::new(Checkupdates::new()),
            aur_updates: Arc::new(Paru::new()),
            merges: Arc::new(Pacdiff::new()),
            update_log: Arc::new(PacmanLog::system()),
            health: Arc::new(GarudaHealth::new()),
            host: Arc::new(SystemHost::from_process()),
        }
    }
}

/// Every status probe at once, each on its own thread, so the slowest one
/// sets the time instead of their sum. Blocks.
pub fn probe_status(sources: &StatusSources) -> Status {
    thread::scope(|scope| {
        let updates = scope.spawn(|| sources.updates.pending());
        let aur_updates = scope.spawn(|| sources.aur_updates.pending_aur());
        let merges = scope.spawn(|| sources.merges.waiting());
        let health = scope.spawn(|| probe_health(sources));
        let last_update = sources.update_log.last_update();

        Status {
            updates: joined(updates),
            aur_updates: joined(aur_updates),
            merges: joined(merges),
            last_update,
            health: joined(health),
        }
    })
}

/// garuda-health checks an installed system, the live ISO has nothing to
/// fix. When the live check itself fails, health runs anyway.
fn probe_health(sources: &StatusSources) -> Result<Vec<HealthIssue>, ProbeError> {
    if sources.host.is_live_iso() == Ok(true) {
        tracing::debug!("garuda-health skipped on the live ISO");
        return Err(ProbeError::Unavailable);
    }

    sources.health.issues()
}

/// A probe thread's answer. A panic in an adapter fails only its card.
fn joined<T>(handle: thread::ScopedJoinHandle<'_, Result<T, ProbeError>>) -> Result<T, ProbeError> {
    handle
        .join()
        .unwrap_or_else(|_| Err(ProbeError::Failed("the probe panicked".to_owned())))
}
