use std::time::{Duration, UNIX_EPOCH};

use super::{Style, format_record, timestamp};

#[test]
fn timestamps_are_utc_iso() {
    let time = UNIX_EPOCH + Duration::from_millis(1_791_106_229_123);
    assert_eq!(timestamp(time), "2026-10-04T09:30:29.123Z");
    assert_eq!(timestamp(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
}

#[test]
fn timestamps_get_leap_days_and_century_years_right() {
    for (seconds, expected) in [
        (1_709_251_199, "2024-02-29T23:59:59.000Z"),
        (951_825_600, "2000-02-29T12:00:00.000Z"),
        (4_107_542_400, "2100-03-01T00:00:00.000Z"),
        (946_684_799, "1999-12-31T23:59:59.000Z"),
    ] {
        assert_eq!(
            timestamp(UNIX_EPOCH + Duration::from_secs(seconds)),
            expected
        );
    }
}

#[test]
fn terminal_lines_are_coloured_by_severity() {
    let line = format_record(UNIX_EPOCH, log::Level::Error, "hub", "boom", Style::Color);

    assert_eq!(
        line,
        "\x1b[2m1970-01-01T00:00:00.000Z\x1b[0m \x1b[1;31mERROR\x1b[0m \x1b[36mhub\x1b[0m: boom\n"
    );
    assert!(
        format_record(UNIX_EPOCH, log::Level::Info, "x", "y", Style::Plain)
            .chars()
            .all(|ch| ch != '\x1b')
    );
}

#[test]
fn multi_line_messages_are_indented() {
    let line = format_record(
        UNIX_EPOCH,
        log::Level::Warn,
        "nix",
        "failed\nerror: x\n",
        Style::Plain,
    );

    assert_eq!(
        line,
        "1970-01-01T00:00:00.000Z WARN  nix: failed\n    error: x\n"
    );
}

#[test]
fn messages_keep_no_control_characters() {
    let line = format_record(
        UNIX_EPOCH,
        log::Level::Info,
        "apply",
        "step said \u{1b}[2Kdone\u{7}",
        Style::Plain,
    );

    assert_eq!(
        line,
        "1970-01-01T00:00:00.000Z INFO  apply: step said [2Kdone\n"
    );
}
