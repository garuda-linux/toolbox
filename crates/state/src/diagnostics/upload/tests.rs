use strum::IntoEnumIterator as _;

use super::UploadFailure;

#[test]
fn every_failure_has_its_stable_code() {
    let codes: Vec<&str> = UploadFailure::iter().map(UploadFailure::as_str).collect();

    assert_eq!(
        codes,
        ["network", "refused", "bad-answer", "seal", "stopped"]
    );
}
