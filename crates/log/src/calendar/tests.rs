use super::{Utc, civil_from_days, days_from_civil};

#[test]
fn the_epoch() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(days_from_civil(1970, 1, 1), 0);
}

#[test]
fn a_leap_day() {
    assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    assert_eq!(days_from_civil(2000, 2, 29), 11_016);
}

#[test]
fn the_two_conversions_round_trip() {
    for day in [0, 1, 11_015, 19_723, 20_454, 29_200] {
        let (year, month, date) = civil_from_days(day);
        assert_eq!(days_from_civil(year, month, date), day);
    }
}

#[test]
fn days_before_the_epoch_are_negative() {
    assert_eq!(days_from_civil(1969, 12, 31), -1);
    assert_eq!(civil_from_days(-1), (1969, 12, 31));
}

#[test]
fn a_unix_time_splits_into_date_and_clock() {
    assert_eq!(
        Utc::from_unix(1_709_251_199),
        Utc {
            year: 2024,
            month: 2,
            day: 29,
            hour: 23,
            minute: 59,
            second: 59,
        }
    );
}
