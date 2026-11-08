#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Upgrader {
    /// Garuda's wrapper: snapshots, keyrings and hooks included. Mirrors
    /// are refreshed on their own, so it skips them.
    GarudaUpdate,
    /// Plain pacman on systems without garuda-update.
    Pacman,
}

/// The repair scripts `garuda-update remote` fetches and runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAction {
    Fix,
    Keyring,
    FullFix,
    ResetAudio,
    ResetSnapper,
    Reinstall,
}

impl RemoteAction {
    pub const ALL: [Self; 6] = [
        Self::Fix,
        Self::Keyring,
        Self::FullFix,
        Self::ResetAudio,
        Self::ResetSnapper,
        Self::Reinstall,
    ];

    /// The argument garuda-update takes, also the stable id for the UI.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fix => "fix",
            Self::Keyring => "keyring",
            Self::FullFix => "fullfix",
            Self::ResetAudio => "reset-audio",
            Self::ResetSnapper => "reset-snapper",
            Self::Reinstall => "reinstall",
        }
    }
}
