//! How often the system status is checked again: `"off"`, or minutes
//! written as `"45m"`, whole hours as `"3h"`, so the files older versions
//! wrote (`30m`, `1h`, `3h`, `6h`) read and write unchanged.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// The shortest period, so a check never runs back to back.
pub const MIN_MINUTES: u16 = 5;
/// The longest period, a day.
pub const MAX_MINUTES: u16 = 24 * PER_HOUR;

const PER_HOUR: u16 = 60;
/// The period a new install checks at, an hour.
pub const DEFAULT_MINUTES: u16 = PER_HOUR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshInterval {
    /// Only when the user asks.
    Off,
    /// Every this many minutes, from [`MIN_MINUTES`] to [`MAX_MINUTES`].
    Minutes(u16),
}

impl Default for RefreshInterval {
    fn default() -> Self {
        Self::Minutes(DEFAULT_MINUTES)
    }
}

impl RefreshInterval {
    /// The time between two checks, `None` when only the user starts one.
    pub fn period(self) -> Option<Duration> {
        match self {
            Self::Off => None,
            Self::Minutes(minutes) => Some(Duration::from_mins(u64::from(minutes))),
        }
    }

    /// `minutes` held to [`MIN_MINUTES`] and [`MAX_MINUTES`].
    pub fn clamped(minutes: u32) -> Self {
        let held = minutes.clamp(u32::from(MIN_MINUTES), u32::from(MAX_MINUTES));
        Self::Minutes(u16::try_from(held).unwrap_or(MAX_MINUTES))
    }

    fn parse(text: &str) -> Option<Self> {
        if text == "off" {
            return Some(Self::Off);
        }

        let (number, factor) = if let Some(hours) = text.strip_suffix('h') {
            (hours, PER_HOUR)
        } else {
            (text.strip_suffix('m')?, 1)
        };

        let minutes = number.parse::<u16>().ok()?.checked_mul(factor)?;
        (MIN_MINUTES..=MAX_MINUTES)
            .contains(&minutes)
            .then_some(Self::Minutes(minutes))
    }
}

impl fmt::Display for RefreshInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Off => f.write_str("off"),
            Self::Minutes(minutes) if minutes % PER_HOUR == 0 => {
                write!(f, "{}h", minutes / PER_HOUR)
            }
            Self::Minutes(minutes) => write!(f, "{minutes}m"),
        }
    }
}

impl Serialize for RefreshInterval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for RefreshInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).ok_or_else(|| {
            de::Error::custom(format!(
                "\"{text}\" is no refresh interval: \"off\", or {MIN_MINUTES} to {MAX_MINUTES} \
                 minutes such as \"45m\" or \"3h\""
            ))
        })
    }
}
