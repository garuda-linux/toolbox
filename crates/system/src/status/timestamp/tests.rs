use std::time::{Duration, SystemTime};

use super::parse_timestamp;

fn at(seconds: u64) -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

#[test]
fn an_offset_is_subtracted() {
    assert_eq!(
        parse_timestamp("2026-10-09T14:03:11+0200"),
        at(1_791_547_391)
    );
    assert_eq!(
        parse_timestamp("2026-10-09T12:03:11+0000"),
        at(1_791_547_391)
    );
}

#[test]
fn a_negative_offset_with_minutes_is_added() {
    assert_eq!(parse_timestamp("2000-02-29T00:00:00-0530"), at(951_802_200));
}

#[test]
fn rfc_3339_offsets_and_zulu_are_read_too() {
    assert_eq!(
        parse_timestamp("2026-10-09T14:03:11+02:00"),
        at(1_791_547_391)
    );
    assert_eq!(parse_timestamp("2026-10-09T14:03:11Z"), at(1_791_554_591));
}

/// pacman before 5.1 wrote local time to the minute, without a zone.
#[test]
fn the_old_format_is_read_as_utc() {
    assert_eq!(parse_timestamp("2024-01-02 10:11"), at(1_704_190_260));
}

#[test]
fn the_epoch_itself_is_valid() {
    assert_eq!(parse_timestamp("1970-01-01T00:00:00+0000"), at(0));
}

#[test]
fn invalid_dates_and_times_are_rejected() {
    for text in [
        "",
        "garbage",
        "2026-13-01T00:00:00+0000",
        "2026-02-30T00:00:00+0000",
        "2025-02-29T00:00:00+0000",
        "2026-10-09T24:00:00+0000",
        "2026-10-09T14:60:00+0000",
        "2026-10-09T14:03:11",
        "2026-10-09T14:03:11+02",
        "2026-10-09T14:03:11+2400",
        "2026-10-09 14:03:11",
        "2026-10-09T14:03:11+0200 extra",
        "26-10-09T14:03:11+0200",
        "2026-1-09T14:03:11+0200",
        "+026-10-09T14:03:11+0200",
    ] {
        assert_eq!(parse_timestamp(text), None, "{text}");
    }
}

#[test]
fn leap_days_are_counted() {
    assert_eq!(parse_timestamp("2024-02-29 00:00"), at(1_709_164_800));
    assert_eq!(parse_timestamp("2024-03-01 00:00"), at(1_709_251_200));
}
