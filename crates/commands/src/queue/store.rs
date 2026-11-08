//! Where the queue lives for a command: in the running window, which
//! alone writes `changes.json` while it runs, or in that file when no
//! window runs.

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use tokio::runtime::Handle;
use toolbox_core::{SAVED_VERSION, SavedChanges, Snapshot, domains};
use toolbox_instance::bus::queue::{Refusal, Window};
use toolbox_services::queue::edit::QueueEdit;
use toolbox_services::queue::{read, store};

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
        Ok(read(&self.path, &Snapshot::default()).to_saved())
    }

    fn edit(&self, edit: &QueueEdit) -> Result<(), Problem> {
        let domains = domains::all().map_err(|error| Problem::File(error.to_string()))?;
        let mut changes = read(&self.path, &Snapshot::default());

        for problem in edit.apply(&mut changes, &domains, &Snapshot::default()) {
            tracing::warn!("dropped a change for the queue: {problem}");
        }

        if let Some(folder) = self.path.parent() {
            std::fs::create_dir_all(folder).map_err(|error| Problem::File(error.to_string()))?;
        }

        store(&self.path, &changes.to_saved()).map_err(|error| Problem::File(error.to_string()))
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
        self.runtime
            .block_on(self.window.edit(&edit.to_text()))
            .map_err(Problem::Window)
    }
}

/// # Errors
///
/// [`Problem::Window`] when the bus cannot say whether a window runs: the
/// file is then left alone, as the window might be writing it.
pub fn open(runtime: &Handle, path: PathBuf) -> Result<Box<dyn Store>, Problem> {
    match runtime.block_on(Window::find()) {
        Ok(Some(window)) => Ok(Box::new(InWindow {
            window,
            runtime: runtime.clone(),
        })),
        Ok(None) => Ok(Box::new(File { path })),
        Err(error) => {
            tracing::warn!("cannot tell whether the window runs: {error}");
            Err(Problem::Window(Refusal::NoAnswer))
        }
    }
}
