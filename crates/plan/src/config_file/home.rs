//! A write of a config file in the user's home. It runs as the user in
//! the app, never through the helper, which has no home.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{ConfigText, Fingerprint, UNCHANGED_WHY};
use crate::HomePath;
use crate::procedure::{Builtin, Invocation};

/// Where a home write keeps the previous version, below
/// `$XDG_STATE_HOME`.
pub const HOME_CONFIG_BACKUPS: &str = "garuda-toolbox/backups";

/// One replacement of a file in the home: which, the new text, and the
/// mark of the version the user started from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeWrite {
    pub path: HomePath,
    pub content: ConfigText,
    pub original: Fingerprint,
}

impl HomeWrite {
    pub fn invocation(&self) -> Invocation {
        Invocation::Builtin(Builtin::WriteHomeFile(self.clone()))
    }

    /// Where the previous version goes, below `$XDG_STATE_HOME`.
    pub fn backup(&self) -> String {
        format!("{HOME_CONFIG_BACKUPS}/{}", self.path.as_str())
    }

    /// The write as the Console shows it, without its note.
    pub fn command(&self) -> String {
        format!(
            "write ~/{} ({} bytes), the old one kept as ~/.local/state/{}",
            self.path.as_str(),
            self.content.as_str().len(),
            self.backup()
        )
    }
}

/// What the Review page and the log show for the write.
impl fmt::Display for HomeWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}  # {UNCHANGED_WHY}", self.command())
    }
}
