//! The rows the Maintenance page offers from `/etc/skel`, and the reset
//! that copies them. `toolbox_services::shelf::Shelf` holds the choice.

use std::path::Path;
use std::time::SystemTime;

use toolbox_catalog::{SkelCatalog, SkelEntry};

/// `files` are full skel paths, for the confirmation that lists what gets replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkelRow {
    pub entry: String,
    pub name: String,
    pub files: Vec<String>,
    /// The same files in the home folder, such as `~/.bashrc`, which a
    /// reset replaces and backs up.
    pub targets: Vec<String>,
}

/// What the last reset did. `done` is false until one ran. `failed` holds
/// the names of the entries that could not be reset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub done: bool,
    pub copied: usize,
    pub failed: Vec<String>,
}

impl Outcome {
    /// A reset that ended without copying: every entry counts as failed.
    pub fn stopped(entries: &[SkelEntry]) -> Self {
        Self {
            done: true,
            copied: 0,
            failed: entries
                .iter()
                .map(|entry| entry.name().to_owned())
                .collect(),
        }
    }
}

/// Copies `entries` from `skel_root` into `home`. Blocks while it copies
/// whole folders, so it runs off the GUI thread.
pub fn reset(entries: &[SkelEntry], skel_root: &Path, home: &Path) -> Outcome {
    let mut outcome = Outcome {
        done: true,
        ..Outcome::default()
    };

    for entry in entries {
        match super::reset(entry, skel_root, home, SystemTime::now()) {
            Ok(report) => {
                outcome.copied = outcome.copied.saturating_add(report.copied.len());
            }
            Err(error) => {
                tracing::error!("cannot reset {}: {error}", entry.id());
                outcome.failed.push(entry.name().to_owned());
            }
        }
    }

    tracing::info!(
        "reset {} settings files from {}",
        outcome.copied,
        skel_root.display()
    );
    outcome
}

pub fn offered(catalog: &SkelCatalog, skel_root: &Path) -> Vec<SkelRow> {
    super::available(catalog.entries(), skel_root)
        .into_iter()
        .map(|entry| SkelRow {
            entry: entry.id().to_owned(),
            name: entry.name().to_owned(),
            files: entry
                .files()
                .iter()
                .map(|file| skel_root.join(file).display().to_string())
                .collect(),
            targets: entry
                .files()
                .iter()
                .map(|file| format!("~/{}", file.display()))
                .collect(),
        })
        .collect()
}
