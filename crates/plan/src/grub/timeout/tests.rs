use super::GrubTimeout;
use crate::{GrubSetting, GrubState};

#[test]
fn minus_one_waits_for_a_key_and_is_written_back_as_minus_one() {
    let wait = GrubTimeout::new(-1).unwrap();

    assert_eq!(wait, GrubTimeout::WaitForKey);
    assert_eq!(wait.seconds(), None);
    assert_eq!(
        GrubSetting::Timeout(wait).apply("GRUB_TIMEOUT=5\n"),
        "GRUB_TIMEOUT=-1\n"
    );
}

#[test]
fn seconds_stay_between_zero_and_three_hundred() {
    assert_eq!(GrubTimeout::new(0).unwrap().seconds(), Some(0));
    assert_eq!(GrubTimeout::new(300).unwrap().seconds(), Some(300));
    GrubTimeout::new(301).unwrap_err();
    GrubTimeout::new(-2).unwrap_err();
}

/// GRUB waits for a key on any negative number, so a file with another
/// one already has what Wait for a key asks.
#[test]
fn any_negative_timeout_in_the_file_meets_wait_for_a_key() {
    let setting = GrubSetting::Timeout(GrubTimeout::WaitForKey);

    assert!(setting.is_met_by(&GrubState::read("GRUB_TIMEOUT=-1\n")));
    assert!(setting.is_met_by(&GrubState::read("GRUB_TIMEOUT=-5\n")));
    assert!(!setting.is_met_by(&GrubState::read("GRUB_TIMEOUT=0\n")));
}

#[test]
fn the_helper_reads_back_what_the_app_sent() {
    for timeout in [GrubTimeout::WaitForKey, GrubTimeout::new(12).unwrap()] {
        let sent = serde_json::to_string(&timeout).unwrap();

        assert_eq!(serde_json::from_str::<GrubTimeout>(&sent).unwrap(), timeout);
    }

    assert_eq!(
        serde_json::to_string(&GrubTimeout::WaitForKey).unwrap(),
        "-1"
    );
    serde_json::from_str::<GrubTimeout>("-3").unwrap_err();
}

#[test]
fn a_file_value_becomes_the_nearest_timeout_the_page_offers() {
    assert_eq!(GrubTimeout::nearest(-5), GrubTimeout::WaitForKey);
    assert_eq!(GrubTimeout::nearest(900).seconds(), Some(300));
    assert_eq!(GrubTimeout::nearest(7).seconds(), Some(7));
    assert_eq!(GrubTimeout::default().seconds(), Some(5));
}
