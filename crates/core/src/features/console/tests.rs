use strum::IntoEnumIterator as _;

use super::HelperFailure;

#[test]
fn every_failure_has_its_stable_code() {
    let codes: Vec<&str> = HelperFailure::iter().map(HelperFailure::as_str).collect();

    assert_eq!(
        codes,
        [
            "missing-secret",
            "start",
            "pkexec",
            "no-pipes",
            "lost",
            "ended",
            "user-steps"
        ]
    );
}
