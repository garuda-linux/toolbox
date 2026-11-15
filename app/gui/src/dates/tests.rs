use egui::Context;

use super::{day, moment, pacman_moment};
use crate::i18n;

/// 2026-11-14 12:00 UTC, the same day in every time zone from UTC-11 to UTC+11.
const NOON: i64 = 1_794_657_600;

fn in_language(tag: &str) -> Context {
    let ctx = Context::default();
    i18n::set_language(&ctx, &tag.parse().expect("a language tag"));
    ctx
}

#[test]
fn the_day_takes_the_language_medium_format() {
    assert_eq!(day(&in_language("en-US"), NOON), "Nov 14, 2026");
    assert_eq!(day(&in_language("de-DE"), NOON), "14.11.2026");
}

/// Whole-hour zones show the same minutes, so the test holds in any of them.
#[test]
fn a_moment_takes_the_language_short_format() {
    let english = moment(&in_language("en-US"), NOON).expect("a moment");
    let german = moment(&in_language("de-DE"), NOON).expect("a moment");
    assert!(english.contains(":00"), "{english}");
    assert!(german.contains(":00"), "{german}");
    assert!(!german.contains("PM"), "{german}");
    assert_ne!(english, german);
}

#[test]
fn pacman_writes_the_same_moment_with_its_offset() {
    let ctx = in_language("de-DE");
    let utc = pacman_moment(&ctx, "2026-11-14T12:00:00+0000");
    let zone = pacman_moment(&ctx, "2026-11-14T14:00:00+0200");
    assert_eq!(utc, moment(&ctx, NOON));
    assert_eq!(zone, moment(&ctx, NOON));
}

#[test]
fn an_old_pacman_line_without_a_zone_reads_as_local_time() {
    let ctx = in_language("de-DE");
    let line = pacman_moment(&ctx, "2026-11-14 12:00").expect("a moment");
    assert!(line.contains("14.11.26"), "{line}");
}

#[test]
fn other_text_is_no_moment() {
    assert_eq!(pacman_moment(&in_language("en-US"), "yesterday"), None);
}
