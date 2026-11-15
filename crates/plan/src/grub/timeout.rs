//! `GRUB_TIMEOUT`: the seconds the menu waits, or a menu that waits until
//! a key is pressed.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Bounded, OutOfRange};

/// The number GRUB reads as "wait for a key". GRUB takes any negative one
/// alike, the page writes this one.
const WAIT: i64 = -1;
/// How long GRUB waits when the file names no timeout.
const DEFAULT_SECONDS: i64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub enum GrubTimeout {
    Seconds(Bounded<0, 300>),
    WaitForKey,
}

impl GrubTimeout {
    /// # Errors
    ///
    /// [`OutOfRange`] for a number other than -1 outside 0 to 300.
    pub fn new(value: i64) -> Result<Self, OutOfRange> {
        if value == WAIT {
            return Ok(Self::WaitForKey);
        }

        Bounded::new(value).map(Self::Seconds)
    }

    /// What GRUB does with `value` in its file, past 300 seconds cut to
    /// what the page offers.
    pub fn nearest(value: i64) -> Self {
        if value.is_negative() {
            return Self::WaitForKey;
        }

        Self::Seconds(Bounded::nearest(value))
    }

    pub const fn seconds(self) -> Option<u32> {
        match self {
            Self::Seconds(seconds) => Some(seconds.get()),
            Self::WaitForKey => None,
        }
    }

    /// Whether the file's `GRUB_TIMEOUT` already has this.
    pub fn is_met_by(self, file: Option<i64>) -> bool {
        match self {
            Self::Seconds(seconds) => file == Some(i64::from(seconds.get())),
            Self::WaitForKey => file.is_some_and(i64::is_negative),
        }
    }
}

/// GRUB's own timeout when the file sets none.
impl Default for GrubTimeout {
    fn default() -> Self {
        Self::Seconds(Bounded::nearest(DEFAULT_SECONDS))
    }
}

impl TryFrom<i64> for GrubTimeout {
    type Error = OutOfRange;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GrubTimeout> for i64 {
    fn from(value: GrubTimeout) -> Self {
        value.seconds().map_or(WAIT, Self::from)
    }
}

impl fmt::Display for GrubTimeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        i64::from(*self).fmt(f)
    }
}
