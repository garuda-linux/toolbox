//! Timestamps of `/var/log/pacman.log`, without a date crate.
//!
//! pacman 5.1 and later write `2026-10-09T14:03:11+0200` (local time with
//! its offset). Older logs hold `2024-01-02 10:11`: local time to the
//! minute, without a zone.

#[cfg(test)]
mod tests;

use std::time::{Duration, SystemTime};

use toolbox_calendar::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, days_from_civil};

/// The moment `text` names, `None` when it is no timestamp of either
/// format. RFC 3339 offsets (`+02:00`, `Z`) are read too.
///
/// The old format has no zone. It is read as UTC: without a time zone
/// database the local offset is unknown, and a few hours do not matter
/// for "updated in the last two weeks".
pub fn parse_timestamp(text: &str) -> Option<SystemTime> {
    let (date, clock, offset) = if let Some((date, rest)) = text.split_once('T') {
        let zone_at = rest.find(['+', '-', 'Z'])?;
        let (clock, zone) = rest.split_at_checked(zone_at)?;

        (date, seconds_of_day(clock, 3)?, offset_seconds(zone)?)
    } else {
        let (date, clock) = text.split_once(' ')?;

        (date, seconds_of_day(clock, 2)?, 0)
    };

    let days = days_since_epoch(date)?;

    let seconds = days
        .checked_mul(i64::from(SECONDS_PER_DAY))?
        .checked_add(clock)?
        .checked_sub(offset)?;

    u64::try_from(seconds).map_or_else(
        |_| SystemTime::UNIX_EPOCH.checked_sub(Duration::from_secs(seconds.unsigned_abs())),
        |after| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(after)),
    )
}

/// Days from 1970-01-01 to `YYYY-MM-DD`.
fn days_since_epoch(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let year = number(parts.next()?, 4)?;
    let month = number(parts.next()?, 2)?;
    let day = number(parts.next()?, 2)?;

    if parts.next().is_some() || day == 0 || day > days_in_month(year, month)? {
        return None;
    }

    Some(days_from_civil(year, month, day))
}

const fn days_in_month(year: i64, month: i64) -> Option<i64> {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;

    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if leap => Some(29),
        2 => Some(28),
        _ => None,
    }
}

/// `HH:MM:SS` (`fields` 3) or `HH:MM` (`fields` 2) as seconds.
fn seconds_of_day(clock: &str, fields: usize) -> Option<i64> {
    let values: Vec<i64> = clock
        .split(':')
        .map(|part| number(part, 2))
        .collect::<Option<_>>()?;

    if values.len() != fields {
        return None;
    }

    let mut parts = values.into_iter();
    let hours = parts.next()?;
    let minutes = parts.next()?;
    let seconds = parts.next().unwrap_or(0);

    (hours < 24 && minutes < 60 && seconds < 60)
        .then_some(hours_and_minutes(hours, minutes) + seconds)
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

    let hours = number(hours, 2)?;
    let minutes = number(minutes, 2)?;

    (hours < 24 && minutes < 60).then_some(sign * hours_and_minutes(hours, minutes))
}

/// Seconds of `hours` and `minutes`.
fn hours_and_minutes(hours: i64, minutes: i64) -> i64 {
    hours * i64::from(SECONDS_PER_HOUR) + minutes * i64::from(SECONDS_PER_MINUTE)
}

/// Exactly `width` ASCII digits.
fn number(text: &str, width: usize) -> Option<i64> {
    (text.len() == width && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}
