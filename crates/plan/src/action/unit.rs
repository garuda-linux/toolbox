//! What a unit action does to its units and which service manager owns
//! them.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

/// A one-off operation on running units. Enabling and masking are states
/// and go through the queue instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitOperation {
    Start,
    Stop,
    Restart,
    Reload,
}

impl UnitOperation {
    /// The systemctl verb, also the stable id for the UI.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Reload => "reload",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitScope {
    System,
    User,
}

impl UnitScope {
    /// The user's own units for `user`, else the system's, as the
    /// `--user` flag and the Services setting say it.
    pub const fn from_user(user: bool) -> Self {
        if user { Self::User } else { Self::System }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
        }
    }
}
