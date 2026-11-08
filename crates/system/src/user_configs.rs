//! Whether the user config edits of the catalogs are on, read from the
//! files in the user's home. Only the files the catalogs name are read, so
//! the probe stays as cheap as a few small reads.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::{fs, io};

use toolbox_plan::UserConfigEdit;
use toolbox_state::parts::UserConfigStates;

use crate::error::ProbeError;

/// Answers which config edits are on.
pub trait UserConfigSource: Send + Sync {
    /// The state of each edit in `edits`. An edit whose file cannot be read
    /// is left out, so its entry shows as not known.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a home, when there are edits to
    /// read.
    fn states(&self, edits: &[UserConfigEdit]) -> Result<UserConfigStates, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeConfigs {
    home: Option<PathBuf>,
}

impl HomeConfigs {
    /// The home of the user running the app, from `HOME`.
    #[must_use = "the adapter does nothing until asked"]
    pub fn from_env() -> Self {
        Self {
            home: std::env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .map(PathBuf::from),
        }
    }

    #[must_use = "the adapter does nothing until asked"]
    #[cfg(test)]
    pub(crate) fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            home: Some(home.into()),
        }
    }
}

impl UserConfigSource for HomeConfigs {
    fn states(&self, edits: &[UserConfigEdit]) -> Result<UserConfigStates, ProbeError> {
        if edits.is_empty() {
            return Ok(UserConfigStates::default());
        }

        let home = self.home.as_ref().ok_or(ProbeError::Unavailable)?;
        let mut states = BTreeMap::new();

        for edit in edits {
            let path = home.join(edit.path().as_str());

            match fs::read(&path) {
                Ok(bytes) => {
                    states.insert(edit.clone(), edit.is_on(Some(&bytes)));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    states.insert(edit.clone(), edit.is_on(None));
                }
                Err(error) => tracing::debug!("{} is not readable: {error}", path.display()),
            }
        }

        Ok(UserConfigStates(states))
    }
}
