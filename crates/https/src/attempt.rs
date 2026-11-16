//! The refresh rule of a cache: one download a period, with the time of
//! every attempt stored before the request goes out. An attempt that
//! fails, hangs or crashes then still holds the next one back for the
//! whole period, and a caller that cannot store the time does not ask.
//! An attempt stored in the future, from a clock set ahead, counts as now,
//! or every download would wait until that time passed.

use std::time::Duration;

/// What the stored time of the last attempt allows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attempt {
    /// The stored time lies after `now`. Store `now` and do not ask.
    ClockAhead,
    /// The period has not passed since the stored time.
    NotDue,
    /// No attempt is stored or the period has passed. Store `now`, then ask.
    Due,
}

/// `last` is the stored time of the last attempt in seconds since the
/// epoch, `None` when there was none.
pub fn due(last: Option<i64>, now: i64, every: Duration) -> Attempt {
    match last {
        Some(last) if last > now => Attempt::ClockAhead,
        Some(last) if now < next_attempt(last, every) => Attempt::NotDue,
        _ => Attempt::Due,
    }
}

/// The first time an attempt is due again after the one stored as `last`.
pub fn next_attempt(last: i64, every: Duration) -> i64 {
    let period = i64::try_from(every.as_secs()).unwrap_or(i64::MAX);
    last.saturating_add(period)
}
