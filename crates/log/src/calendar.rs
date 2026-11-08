//! Proleptic Gregorian calendar arithmetic in UTC.
//!
//! After Howard Hinnant's `days_from_civil` and `civil_from_days`, without
//! a date crate or a time zone database. Shared by the log's timestamps,
//! the backup names and the pacman log timestamps.

#[cfg(test)]
mod tests;

pub const SECONDS_PER_MINUTE: u32 = 60;
pub const SECONDS_PER_HOUR: u32 = 60 * SECONDS_PER_MINUTE;
pub const SECONDS_PER_DAY: u32 = 24 * SECONDS_PER_HOUR;

/// Days from 0000-03-01 to 1970-01-01.
const EPOCH_DAYS: i64 = 719_468;

/// Days in one 400 year cycle.
const DAYS_PER_ERA: i64 = 146_097;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utc {
    pub year: i64,
    pub month: i64,
    pub day: i64,
    pub hour: u64,
    pub minute: u64,
    pub second: u64,
}

impl Utc {
    pub fn from_unix(seconds: u64) -> Self {
        let per_day = u64::from(SECONDS_PER_DAY);
        let per_hour = u64::from(SECONDS_PER_HOUR);
        let per_minute = u64::from(SECONDS_PER_MINUTE);

        let days = i64::try_from(seconds / per_day).unwrap_or(i64::MAX);
        let (year, month, day) = civil_from_days(days);
        let of_day = seconds % per_day;

        Self {
            year,
            month,
            day,
            hour: of_day / per_hour,
            minute: of_day % per_hour / per_minute,
            second: of_day % per_minute,
        }
    }
}

/// Year, month and day of the day `days` after 1970-01-01. The year starts
/// in March, so the leap day ends it.
pub const fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let since_era_zero = days + EPOCH_DAYS;
    let era = since_era_zero.div_euclid(DAYS_PER_ERA);
    let day_of_era = since_era_zero - era * DAYS_PER_ERA;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;

    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);

    // Months counted from March, each run of five months has 153 days.
    let march_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * march_month + 2) / 5 + 1;
    let month = if march_month < 10 {
        march_month + 3
    } else {
        march_month - 9
    };

    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

/// Days from 1970-01-01 to `year-month-day` in the proleptic Gregorian
/// calendar.
pub const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * DAYS_PER_ERA + day_of_era - EPOCH_DAYS
}
