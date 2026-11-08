use std::time::{Duration, SystemTime};

use super::parse_timestamp;

/// The stamp of the first line of `/var/log/pacman.log` in the Lima VM.
const LIMA: &str = "2026-11-08T17:30:04+0100";
const LIMA_AT: u64 = 1_794_155_404;

fn at(seconds: u64) -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

#[test]
fn real_stamps_are_the_ones_of_the_log() {
    let log = include_str!("../pacman_log/fixtures/pacman.log");

    assert!(log.starts_with(&format!("[{LIMA}]")));
}

/// The Lima stamp, and the same moment written in UTC.
#[test]
fn an_offset_is_subtracted() {
    assert_eq!(parse_timestamp(LIMA), at(LIMA_AT));
    assert_eq!(
        parse_timestamp(&LIMA.replacen("17:30:04+0100", "16:30:04+0000", 1)),
        at(LIMA_AT)
    );
}

/// No captured log has a negative offset, this one is made up.
#[test]
fn a_negative_offset_with_minutes_is_added() {
    assert_eq!(parse_timestamp("2000-02-29T00:00:00-0530"), at(951_802_200));
}

/// The Lima stamp with its offset written the RFC 3339 way, and as UTC.
#[test]
fn rfc_3339_offsets_and_zulu_are_read_too() {
    assert_eq!(
        parse_timestamp(&LIMA.replacen("+0100", "+01:00", 1)),
        at(LIMA_AT)
    );
    assert_eq!(
        parse_timestamp(&LIMA.replacen("17:30:04+0100", "16:30:04Z", 1)),
        at(LIMA_AT)
    );
}

/// pacman before 5.1 wrote local time to the minute, without a zone. No
/// captured log is that old, so the Lima stamp is cut to that format.
#[test]
fn the_old_format_is_read_as_utc() {
    assert_eq!(
        parse_timestamp(LIMA.replacen('T', " ", 1).get(..16).unwrap()),
        at(1_794_159_000)
    );
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

/// Leap days in the old format, made up like the old stamps.
#[test]
fn leap_days_are_counted() {
    assert_eq!(parse_timestamp("2024-02-29 00:00"), at(1_709_164_800));
    assert_eq!(parse_timestamp("2024-03-01 00:00"), at(1_709_251_200));
}
