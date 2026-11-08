//! The UI names a failed probe by its code, so codes stay stable and unique.

use std::collections::BTreeSet;

use strum::IntoEnumIterator as _;

use super::{FailedProbes, Probe};

#[test]
fn a_listed_probe_failed_and_the_rest_did_not() {
    let failed = FailedProbes::new([Probe::Hblock, Probe::Accounts]);

    assert!(failed.contains(Probe::Hblock));
    assert!(failed.contains(Probe::Accounts));
    assert!(!failed.contains(Probe::WifiBackend));
    assert!(!FailedProbes::default().contains(Probe::Hblock));
}

#[test]
fn every_probe_has_a_unique_kebab_case_code() {
    let codes: BTreeSet<&str> = Probe::iter().map(Probe::as_str).collect();

    assert_eq!(codes.len(), Probe::iter().count());
    assert_eq!(Probe::WifiBackend.as_str(), "wifi-backend");
    assert_eq!(Probe::LiveIso.as_str(), "live-iso");
    assert_eq!(Probe::SyncPackages.as_str(), "sync-packages");
    assert_eq!(Probe::MaskedUserUnits.as_str(), "masked-user-units");
    assert!(codes.iter().all(|code| {
        code.chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
    }));
}
