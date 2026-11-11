#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};

use toolbox_calendar::Utc;

/// `now` in UTC as `YYYYMMDD-HHMMSS`. A time before 1970 counts as the
/// epoch.
pub fn backup_stamp(now: SystemTime) -> String {
    let seconds = now
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());

    let Utc {
        year,
        month,
        day,
        hour,
        minute,
        second,
    } = Utc::from_unix(seconds);

    format!("{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}")
}
