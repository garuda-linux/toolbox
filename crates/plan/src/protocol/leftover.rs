//! What a failed cleanup of a step left behind, so the next repair's
//! refusal and the Console name the cause.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::repair::{REPAIR_MAPPER, RepairMount};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "left", rename_all = "kebab-case")]
pub enum Leftover {
    /// The unmount failed, so something is still mounted here.
    Mounted { place: RepairMount },
    /// The encrypted system is still unlocked.
    Open,
}

impl fmt::Display for Leftover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mounted { place } => write!(f, "{place} is still mounted"),
            Self::Open => write!(f, "/dev/mapper/{REPAIR_MAPPER} is still open"),
        }
    }
}
