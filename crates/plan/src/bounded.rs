//! A whole number in a fixed range, for the counts and timeouts programs
//! take as arguments.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{value} is not between {min} and {max}")]
pub struct OutOfRange {
    pub value: i64,
    pub min: u32,
    pub max: u32,
}

/// A whole number from `MIN` to `MAX`, so a program never gets a count it
/// would refuse or run forever with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "u32")]
pub struct Bounded<const MIN: u32, const MAX: u32>(u32);

impl<const MIN: u32, const MAX: u32> Bounded<MIN, MAX> {
    pub const MIN: u32 = MIN;
    pub const MAX: u32 = MAX;

    /// # Errors
    ///
    /// [`OutOfRange`] when `value` is below `MIN` or above `MAX`.
    pub fn new(value: i64) -> Result<Self, OutOfRange> {
        u32::try_from(value)
            .ok()
            .filter(|number| (MIN..=MAX).contains(number))
            .map(Self)
            .ok_or(OutOfRange {
                value,
                min: MIN,
                max: MAX,
            })
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<const MIN: u32, const MAX: u32> TryFrom<i64> for Bounded<MIN, MAX> {
    type Error = OutOfRange;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MIN: u32, const MAX: u32> From<Bounded<MIN, MAX>> for u32 {
    fn from(value: Bounded<MIN, MAX>) -> Self {
        value.0
    }
}

impl<const MIN: u32, const MAX: u32> fmt::Display for Bounded<MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
