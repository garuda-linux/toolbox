//! Programs that read their config only at start or on request, told to
//! read it again after an edit. A closed set, so a catalog can name one but
//! never pass a command.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::procedure::{Command, Program};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigReload {
    /// KWin reads `kwinrc` again on its `reloadConfig` signal, the same one
    /// Plasma's settings send.
    Kwin,
}

impl ConfigReload {
    pub const ALL: [Self; 1] = [Self::Kwin];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kwin => "kwin",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|reload| reload.as_str() == id)
    }

    /// dbus-send ships with dbus itself, so no extra tool is needed.
    pub fn command(self) -> Command {
        match self {
            Self::Kwin => Command::new(Program::DbusSend).args([
                "--session",
                "--type=signal",
                "/KWin",
                "org.kde.KWin.reloadConfig",
            ]),
        }
    }
}
