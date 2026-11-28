#[cfg(test)]
mod tests;

use jiff::Timestamp;
use jiff::tz::TimeZone;

/// `now` in UTC as `YYYYMMDD-HHMMSS`. A time before 1970 counts as the
/// epoch.
pub(super) fn backup_stamp(now: Timestamp) -> String {
    let utc = now
        .max(Timestamp::UNIX_EPOCH)
        .to_zoned_datetime(TimeZone::UTC);

    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        utc.year(),
        utc.month(),
        utc.day(),
        utc.hour(),
        utc.minute(),
        utc.second()
    )
}
