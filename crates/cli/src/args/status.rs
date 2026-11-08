//! `status`: the facts once, or again whenever they change.

#[cfg(test)]
mod tests;

use std::time::Duration;

use clap::Args;
use jiff::SignedDuration;

use super::Read;

/// The shortest wait between two looks: each one runs pacman and paru.
const SHORTEST: Duration = Duration::from_secs(1);

/// `status`: the facts, or one line for a login message.
#[derive(Clone, Copy, Debug, Default, Args)]
pub struct StatusArgs {
    /// One line, such as "2 updates, restart needed" or "all good".
    #[arg(long)]
    pub brief: bool,

    /// Look again and again, and print the facts each time they changed,
    /// under the time. Ctrl+C ends it.
    #[arg(long)]
    pub watch: bool,

    /// How long to wait between two looks, such as 30s, 5m or 1h30m.
    #[arg(
        long,
        value_name = "DURATION",
        value_parser = interval,
        default_value = "60s",
        requires = "watch"
    )]
    pub interval: Interval,

    #[command(flatten)]
    pub read: Read,
}

/// The wait between two looks of `status --watch`, at least a second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval(Duration);

impl Interval {
    pub const fn duration(self) -> Duration {
        self.0
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self(Duration::from_mins(1))
    }
}

fn interval(text: &str) -> Result<Interval, String> {
    let parsed: SignedDuration = text
        .parse()
        .map_err(|_unreadable| format!("{text:?} is no duration, write it as 30s, 5m or 1h30m"))?;

    match Duration::try_from(parsed) {
        Ok(wait) if wait >= SHORTEST => Ok(Interval(wait)),
        Ok(_) | Err(_) => Err("wait at least 1s between two looks".to_owned()),
    }
}
