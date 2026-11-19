//! The copies among a self-test's own config files, which `pacdiff -o`
//! cannot list, as it reads the real system's pacman database.

use std::fs;
use std::path::PathBuf;

use super::MergeSource;
use crate::error::{ProbeError, from_io};

const FOLDER: &str = "etc";
const SUFFIXES: [&str; 3] = [".pacnew", ".pacsave", ".pacorig"];

/// Lists the copies directly in `etc` below `root`, as `/etc/<name>`.
pub struct FoundCopies {
    root: PathBuf,
}

impl FoundCopies {
    #[must_use = "the adapter does nothing until asked"]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl MergeSource for FoundCopies {
    fn waiting(&self) -> Result<Vec<PathBuf>, ProbeError> {
        let folder = self.root.join(FOLDER);
        let entries = fs::read_dir(&folder).map_err(|error| from_io(folder.display(), error))?;
        let mut copies: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| SUFFIXES.iter().any(|suffix| name.ends_with(suffix)))
            .map(|name| PathBuf::from(format!("/{FOLDER}/{name}")))
            .collect();

        copies.sort();
        Ok(copies)
    }
}
