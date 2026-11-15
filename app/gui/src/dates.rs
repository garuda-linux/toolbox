//! Dates and times as the language writes them, in the system's time zone.
//! ICU lays out the fields, jiff gives the local wall time.

#[cfg(test)]
mod tests;

use egui::Context;
use icu_calendar::Iso;
use icu_datetime::fieldsets::{YMD, YMDT};
use icu_datetime::input::{Date, DateTime, Time};
use icu_datetime::options::TimePrecision;
use icu_datetime::{DateTimeFormatter, DateTimeFormatterPreferences};
use icu_locale_core::Locale;
use jiff::tz::TimeZone;
use jiff::{Timestamp, civil};

use crate::i18n;

/// The day `seconds` after the epoch falls on, in the system's time zone,
/// in the language's medium date format.
pub fn day(ctx: &Context, seconds: i64) -> String {
    let Some(local) = local(seconds) else {
        return String::new();
    };

    let day = local.date();
    let formatted = iso_date(day).and_then(|date| {
        let formatter = DateTimeFormatter::try_new(preferences(ctx), YMD::medium()).ok()?;
        Some(formatter.format(&date).to_string())
    });

    formatted.unwrap_or_else(|| iso_day(day))
}

/// A moment written as pacman's log writes it, such as
/// `2026-10-09T14:03:11+0200` or `2024-01-02 10:11`, as the language's
/// date and short time in the system's time zone. `None` for any other
/// text.
pub fn pacman_moment(ctx: &Context, text: &str) -> Option<String> {
    let zoned = if text.contains('T') {
        text.parse::<Timestamp>()
            .ok()
            .or_else(|| zone_with_colon(text)?.parse::<Timestamp>().ok())?
            .to_zoned(TimeZone::system())
    } else {
        text.replace(' ', "T")
            .parse::<civil::DateTime>()
            .ok()?
            .to_zoned(TimeZone::system())
            .ok()?
    };

    moment_of(ctx, &zoned.datetime())
}

/// The local date and time of `seconds` after the epoch, in the language's
/// short date and time, to the minute.
pub fn moment(ctx: &Context, seconds: i64) -> Option<String> {
    let local = local(seconds)?;
    moment_of(ctx, &local.datetime())
}

fn iso_date(day: civil::Date) -> Option<Date<Iso>> {
    Date::try_new_iso(
        i32::from(day.year()),
        day.month().unsigned_abs(),
        day.day().unsigned_abs(),
    )
    .ok()
}

fn moment_of(ctx: &Context, time: &civil::DateTime) -> Option<String> {
    let date = iso_date(time.date())?;
    let clock = Time::try_new(
        time.hour().unsigned_abs(),
        time.minute().unsigned_abs(),
        time.second().unsigned_abs(),
        0,
    )
    .ok()?;

    let fields = YMDT::short().with_time_precision(TimePrecision::Minute);
    let formatter = DateTimeFormatter::try_new(preferences(ctx), fields).ok()?;
    let moment = DateTime { date, time: clock };
    Some(formatter.format(&moment).to_string())
}

fn local(seconds: i64) -> Option<jiff::Zoned> {
    Some(
        Timestamp::from_second(seconds)
            .ok()?
            .to_zoned(TimeZone::system()),
    )
}

/// pacman writes the offset as `+0200`, which RFC 3339 spells `+02:00`.
fn zone_with_colon(text: &str) -> Option<String> {
    let (head, zone) = text.split_at_checked(text.len().checked_sub(5)?)?;
    let (sign, digits) = zone.split_at_checked(1)?;
    let (hours, minutes) = digits.split_at_checked(2)?;
    let plain = matches!(sign, "+" | "-")
        && [hours, minutes]
            .iter()
            .all(|part| part.bytes().all(|byte| byte.is_ascii_digit()));

    plain.then(|| format!("{head}{sign}{hours}:{minutes}"))
}

fn preferences(ctx: &Context) -> DateTimeFormatterPreferences {
    let tag = i18n::language(ctx).to_string();
    Locale::try_from_str(&tag).map_or_else(
        |_| DateTimeFormatterPreferences::default(),
        DateTimeFormatterPreferences::from,
    )
}

fn iso_day(date: civil::Date) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}
