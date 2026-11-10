//! Where the queue lives for a command: in the running window, which
//! alone writes `changes.json` while it runs, or in that file when no
//! window runs.

#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::path::PathBuf;

use rustix::fs::{FlockOperation, flock};
use tokio::runtime::Handle;
use toolbox_cli::output::Paint;
use toolbox_core::{ChangeSet, RestoreProblem, SAVED_VERSION, SavedChanges, Snapshot, domains};
use toolbox_instance::bus::queue::{Refusal, Window};
use toolbox_services::queue::edit::QueueEdit;
use toolbox_services::queue::{Restored, keep_aside, read, store};

use super::said::changes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    Window(Refusal),
    /// The file's error, for `--verbose`.
    File(String),
}

pub trait Store {
    fn in_window(&self) -> bool;

    /// # Errors
    ///
    /// [`Problem`] when the queue cannot be read.
    fn saved(&self) -> Result<SavedChanges, Problem>;

    /// # Errors
    ///
    /// [`Problem`] when the queue did not take the edit.
    fn edit(&self, edit: &QueueEdit) -> Result<(), Problem>;
}

pub struct File {
    pub path: PathBuf,
    /// For the warning about changes that could not be restored.
    pub paint: Paint,
}

pub struct InWindow {
    pub window: Window,
    pub runtime: Handle,
}

impl Store for File {
    fn in_window(&self) -> bool {
        false
    }

    /// Restored against no system, so the file's every valid change
    /// comes back. A broken file is an empty queue, as for the window.
    fn saved(&self) -> Result<SavedChanges, Problem> {
        Ok(self.restored().0.to_saved())
    }

    fn edit(&self, edit: &QueueEdit) -> Result<(), Problem> {
        let file = |error: std::io::Error| Problem::File(error.to_string());
        let domains = domains::all().map_err(|error| Problem::File(error.to_string()))?;

        if let Some(folder) = self.path.parent() {
            std::fs::create_dir_all(folder).map_err(file)?;
        }

        let _held = self.lock().map_err(file)?;
        let (mut changes, restored) = self.restored();

        for problem in edit.apply(&mut changes, &domains, &Snapshot::default()) {
            tracing::warn!("dropped a change for the queue: {problem}");
        }

        if restored == Restored::Lossy {
            keep_aside(&self.path).map_err(file)?;
        }

        store(&self.path, &changes.to_saved()).map_err(file)
    }
}

impl File {
    /// The file's queue. What could not be restored is told in one
    /// `warning:` line, as the next write would lose it unseen.
    fn restored(&self) -> (ChangeSet, Restored) {
        let (changes, problems) = read(&self.path, &Snapshot::default());
        if let Some(line) = lost(&problems) {
            writeln!(std::io::stderr(), "{}", self.paint.warning(&line)).ok();
        }

        (changes, Restored::of(&problems))
    }

    /// Held from the read to the write of an edit, so two commands at
    /// once do not lose one's change. The lock file stays, as removing it
    /// would race.
    fn lock(&self) -> std::io::Result<std::fs::File> {
        use std::os::unix::fs::OpenOptionsExt as _;

        let mut name = self.path.as_os_str().to_owned();
        name.push(".lock");

        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .mode(0o600)
            .open(name)?;

        flock(&file, FlockOperation::LockExclusive)?;
        Ok(file)
    }
}

impl Store for InWindow {
    fn in_window(&self) -> bool {
        true
    }

    fn saved(&self) -> Result<SavedChanges, Problem> {
        let text = self
            .runtime
            .block_on(self.window.changes())
            .map_err(Problem::Window)?;

        let saved: SavedChanges = serde_json::from_str(&text).map_err(|error| {
            tracing::warn!("the window's queue came in an unknown shape: {error}");
            Problem::Window(Refusal::Invalid)
        })?;

        if saved.version == SAVED_VERSION {
            Ok(saved)
        } else {
            Err(Problem::Window(Refusal::Unsupported))
        }
    }

    fn edit(&self, edit: &QueueEdit) -> Result<(), Problem> {
        let text = edit.to_text().map_err(|error| {
            tracing::warn!("cannot write the edit for the window: {error}");
            Problem::Window(Refusal::Invalid)
        })?;

        self.runtime
            .block_on(self.window.edit(&text))
            .map_err(Problem::Window)
    }
}

/// # Errors
///
/// [`Problem::Window`] when the bus cannot say whether a window runs: the
/// file is then left alone, as the window might be writing it.
pub fn open(runtime: &Handle, path: PathBuf, paint: Paint) -> Result<Box<dyn Store>, Problem> {
    match runtime.block_on(Window::find()) {
        Ok(Some(window)) => Ok(Box::new(InWindow {
            window,
            runtime: runtime.clone(),
        })),
        Ok(None) => Ok(Box::new(File { path, paint })),
        Err(error) => {
            tracing::warn!("cannot tell whether the window runs: {error}");
            Err(Problem::Window(Refusal::NoAnswer))
        }
    }
}

/// The warning for `problems`, `None` when nothing was lost.
fn lost(problems: &[RestoreProblem]) -> Option<String> {
    let first = problems.first()?;
    let line = match first {
        RestoreProblem::Version(_) | RestoreProblem::Unreadable(_) => {
            format!("the queued changes could not be restored: {first}")
        }
        RestoreProblem::UnknownDomain(_)
        | RestoreProblem::Invalid {
            domain: _,
            reason: _,
        }
        | RestoreProblem::Malformed {
            domain: _,
            error: _,
        } => format!(
            "{} could not be restored, the log says why",
            changes(problems.len())
        ),
    };

    Some(format!(
        "{line}. The old queue is kept in changes.json.bak once it changes"
    ))
}
