//! Timestamps of `/var/log/pacman.log`, read with jiff's civil dates.
//!
//! pacman 5.1 and later write `2026-10-09T14:03:11+0200` (local time with
//! its offset). Older logs hold `2024-01-02 10:11`: local time to the
//! minute, without a zone.

#[cfg(test)]
mod tests;

use std::str::FromStr;

use jiff::civil::DateTime;
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp};

const SECONDS_PER_HOUR: i64 = 3_600;
const SECONDS_PER_MINUTE: i64 = 60;

/// The moment `text` names, `None` when it is no timestamp of either
/// format. RFC 3339 offsets (`+02:00`, `Z`) are read too.
///
/// The old format has no zone. It is read as UTC: without a time zone
/// database the local offset is unknown.
pub(super) fn parse_timestamp(text: &str) -> Option<Timestamp> {
    let (moment, offset) = if let Some((date, rest)) = text.split_once('T') {
        let zone_at = rest.find(['+', '-', 'Z'])?;
        let (clock, zone) = rest.split_at_checked(zone_at)?;

        (civil_moment(date, clock, 3)?, offset_seconds(zone)?)
    } else {
        let (date, clock) = text.split_once(' ')?;

        (civil_moment(date, clock, 2)?, 0)
    };

    moment
        .to_zoned_datetime(TimeZone::UTC)
        .ok()?
        .timestamp()
        .checked_sub(SignedDuration::from_secs(offset))
        .ok()
}

/// The civil moment of `YYYY-MM-DD` and a clock, which jiff checks for
/// days of the month and ranges of the clock.
fn civil_moment(date: &str, clock: &str, fields: usize) -> Option<DateTime> {
    let mut parts = date.split('-');
    let year = number(parts.next()?, 4)?;
    let month = number(parts.next()?, 2)?;
    let day = number(parts.next()?, 2)?;

    if parts.next().is_some() {
        return None;
    }

    let (hour, minute, second) = clock_fields(clock, fields)?;

    DateTime::new(year, month, day, hour, minute, second, 0).ok()
}

/// `HH:MM:SS` (`fields` 3) or `HH:MM` (`fields` 2) as its three fields.
fn clock_fields(clock: &str, fields: usize) -> Option<(i8, i8, i8)> {
    let values: Vec<i8> = clock
        .split(':')
        .map(|part| number(part, 2))
        .collect::<Option<_>>()?;

    if values.len() != fields {
        return None;
    }

    let mut parts = values.into_iter();
    let hour = parts.next()?;
    let minute = parts.next()?;
    let second = parts.next().unwrap_or(0);

    Some((hour, minute, second))
}

/// `Z`, `+HHMM` or `+HH:MM` (also with `-`) as seconds east of UTC.
fn offset_seconds(zone: &str) -> Option<i64> {
    if zone == "Z" {
        return Some(0);
    }

    let (sign, digits) = if let Some(east) = zone.strip_prefix('+') {
        (1, east)
    } else {
        (-1, zone.strip_prefix('-')?)
    };

    let (hours, minutes) = match digits.split_once(':') {
        Some(split) => split,
        None => digits.split_at_checked(2)?,
    };

    let hours: i64 = number(hours, 2)?;
    let minutes: i64 = number(minutes, 2)?;

    let seconds = hours * SECONDS_PER_HOUR + minutes * SECONDS_PER_MINUTE;

    (hours < 24 && minutes < 60).then_some(sign * seconds)
}

/// Exactly `width` ASCII digits, parsed as `T`.
fn number<T>(text: &str, width: usize) -> Option<T>
where
    T: FromStr,
{
    (text.len() == width && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}
