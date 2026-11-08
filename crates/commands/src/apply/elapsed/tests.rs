use std::time::Duration;

use super::elapsed;

fn millis(ms: u64) -> String {
    elapsed(Duration::from_millis(ms))
}

#[test]
fn under_a_minute_shows_tenths_of_a_second() {
    assert_eq!(millis(0), "0.0 s");
    assert_eq!(millis(40), "0.0 s");
    assert_eq!(millis(4_200), "4.2 s");
    assert_eq!(millis(4_249), "4.2 s");
    assert_eq!(millis(4_250), "4.3 s");
    assert_eq!(millis(59_940), "59.9 s");
}

#[test]
fn a_minute_and_more_shows_whole_seconds() {
    assert_eq!(millis(59_950), "1 min");
    assert_eq!(millis(63_000), "1 min 3 s");
    assert_eq!(millis(63_499), "1 min 3 s");
    assert_eq!(millis(120_000), "2 min");
    assert_eq!(millis(3_599_000), "59 min 59 s");
}

#[test]
fn an_hour_and_more_shows_whole_minutes() {
    assert_eq!(millis(3_599_600), "1 h");
    assert_eq!(millis(3_600_000), "1 h");
    assert_eq!(millis(3_900_000), "1 h 5 min");
    assert_eq!(millis(90_000_000), "25 h");
}
