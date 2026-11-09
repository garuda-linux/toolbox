//! The queue's file: read once at start, then written by one task after
//! every change. A restore that lost changes keeps the old file aside
//! before the first write replaces it.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_channel::Receiver;
use tokio::runtime::Handle;
use tokio::sync::watch;
use toolbox_config::write_private;
use toolbox_core::{ChangeSet, RestoreProblem, SavedChanges, Snapshot, domains};

use super::NotKept;
use crate::waking::{self, Wake};

/// How the file came back at start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restored {
    Whole,
    /// Changes were dropped. The old file goes to `changes.json.bak`
    /// before the first write, so what was dropped can still be found.
    Lossy,
}

impl Restored {
    pub const fn of(problems: &[RestoreProblem]) -> Self {
        if problems.is_empty() {
            Self::Whole
        } else {
            Self::Lossy
        }
    }
}

/// The task that writes the file, and what it needs to start once more.
#[derive(Debug)]
pub(super) struct Writer {
    path: PathBuf,
    runtime: Handle,
    wake: Wake,
    newest: watch::Sender<SavedChanges>,
    failed: Receiver<NotKept>,
    /// Shared with the task, so the window's last write on close keeps
    /// the old file aside only when the task has not done so.
    aside: Arc<AtomicBool>,
    restarted: bool,
    /// The second stop was told. A writer that keeps stopping is not
    /// started again, and the toast shows once.
    gone: bool,
}

impl Writer {
    pub(super) fn spawn(
        saved: SavedChanges,
        path: PathBuf,
        runtime: &Handle,
        wake: Wake,
        restored: Restored,
    ) -> Self {
        let aside = Arc::new(AtomicBool::new(restored == Restored::Lossy));
        let (newest, failed) = start(saved, &path, runtime, wake.clone(), &aside);

        Self {
            path,
            runtime: runtime.clone(),
            wake,
            newest,
            failed,
            aside,
            restarted: false,
            gone: false,
        }
    }

    pub(super) fn send(&self, saved: SavedChanges) {
        self.newest.send_replace(saved);
    }

    /// The newest failed write since the last call. A writer that stopped
    /// starts once more and writes what it may have missed. A second stop
    /// is told, once.
    pub(super) fn failure(&mut self) -> Option<NotKept> {
        let failed = self.failed.try_iter().last();
        if self.gone || !self.newest.is_closed() {
            return failed;
        }

        if self.restarted {
            self.gone = true;
            return Some(failed.unwrap_or_else(|| stopped(&self.path)));
        }

        tracing::warn!("the queue writer stopped, starting it once more");
        self.restarted = true;
        let saved = self.newest.borrow().clone();
        let (newest, told) = start(
            saved,
            &self.path,
            &self.runtime,
            self.wake.clone(),
            &self.aside,
        );

        newest.send_modify(|_| {});
        self.newest = newest;
        self.failed = told;
        None
    }

    /// Writes `saved` on the calling thread, for the window's close: the
    /// task may not reach the last change before the runtime stops.
    pub(super) fn flush(&self, saved: &SavedChanges) -> Result<(), NotKept> {
        write(self.path.clone(), saved, &self.aside)
    }
}

fn start(
    saved: SavedChanges,
    path: &Path,
    runtime: &Handle,
    wake: Wake,
    aside: &Arc<AtomicBool>,
) -> (watch::Sender<SavedChanges>, Receiver<NotKept>) {
    let (newest, mut latest) = watch::channel(saved);
    latest.mark_unchanged();
    let (failed, told) = waking::channel(wake);
    let path = path.to_path_buf();
    let aside = Arc::clone(aside);

    drop(runtime.spawn(async move {
        while latest.changed().await.is_ok() {
            let queued = latest.borrow_and_update().clone();
            let target = path.clone();
            let aside = Arc::clone(&aside);
            match tokio::task::spawn_blocking(move || write(target, &queued, &aside)).await {
                Ok(Ok(())) => {}
                Ok(Err(failure)) => {
                    tracing::warn!("{failure}");
                    failed.send(failure);
                }
                Err(_stopped) => {
                    tracing::warn!("the queue writer stopped");
                    failed.send(stopped(&path));
                    return;
                }
            }
        }
    }));

    (newest, told)
}

fn stopped(path: &Path) -> NotKept {
    NotKept {
        path: path.to_path_buf(),
        source: std::io::Error::other("the queue writer stopped"),
    }
}

/// The queue saved at `path`, compared with `snapshot`, and what could not
/// be restored.
///
/// Each problem gets a line in the log. A missing file is an empty queue
/// without a problem. A broken one is an empty queue too: a lost queue is
/// annoying, an app that cannot start is worse.
pub fn read(path: &Path, snapshot: &Snapshot) -> (ChangeSet, Vec<RestoreProblem>) {
    let mut changes = ChangeSet::default();
    let saved = match std::fs::read_to_string(path) {
        Ok(text) => SavedChanges::parse(&text),
        Err(error) if error.kind() == ErrorKind::NotFound => return (changes, Vec::new()),
        Err(error) => Err(RestoreProblem::Unreadable(error.to_string())),
    };

    let problems = match (saved, domains::all()) {
        (Ok(saved), Ok(domains)) => domains.restore(&mut changes, &saved, snapshot),
        (Err(problem), _) => vec![problem],
        (Ok(_), Err(error)) => {
            tracing::error!("the change domains are broken: {error}");
            Vec::new()
        }
    };

    for problem in &problems {
        tracing::warn!("dropped from {}: {problem}", path.display());
    }

    (changes, problems)
}

/// Keeps `saved` at `path` atomically. An empty queue removes the file
/// instead of keeping an empty one. Blocks on two file syncs.
///
/// # Errors
///
/// When the file cannot be written or removed.
pub fn store(path: &Path, saved: &SavedChanges) -> std::io::Result<()> {
    if saved.changes.is_empty() {
        match std::fs::remove_file(path) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error),
            Ok(()) | Err(_) => Ok(()),
        }
    } else {
        let bytes = serde_json::to_vec_pretty(saved).map_err(std::io::Error::other)?;
        write_private(path, &bytes)
    }
}

/// Moves `path` to `<path>.bak`. One generation: a later lossy restore
/// replaces an older backup.
///
/// # Errors
///
/// When the file is there but cannot be moved.
pub fn keep_aside(path: &Path) -> std::io::Result<()> {
    let mut aside = OsString::from(path.as_os_str());
    aside.push(".bak");

    match std::fs::rename(path, aside) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        moved => moved,
    }
}

fn write(path: PathBuf, saved: &SavedChanges, aside: &AtomicBool) -> Result<(), NotKept> {
    // Taken at once, so a second writer never moves the new file aside.
    if aside.swap(false, Ordering::SeqCst)
        && let Err(source) = keep_aside(&path)
    {
        aside.store(true, Ordering::SeqCst);
        return Err(NotKept { path, source });
    }

    store(&path, saved).map_err(|source| NotKept { path, source })
}
