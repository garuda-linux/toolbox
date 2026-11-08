//! systemd-boot's `loader.conf` in the Arch test image, which mounts no
//! ESP: the helper finds none in its own mount table, so the step fails
//! and no file appears at any place an ESP could be.

use std::path::Path;

use toolbox_plan::RootAction;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::sdboot::{LoaderChange, LoaderSetting, LoaderTimeout};

use super::common::TestResult;
use super::{apply, in_test_image};

const PLACES: [&str; 3] = [
    "/efi/loader/loader.conf",
    "/boot/loader/loader.conf",
    "/boot/efi/loader/loader.conf",
];

#[test]
fn without_a_mounted_esp_loader_settings_fail_and_write_nothing() -> TestResult {
    if !in_test_image() || PLACES.iter().any(|place| Path::new(place).exists()) {
        return Ok(());
    }

    let settings = LoaderChange::new(vec![LoaderSetting::Timeout(Some(LoaderTimeout::Seconds(
        5,
    )))])?;

    let outcomes = apply(vec![RootAction::SetLoaderSettings { settings }])?;

    assert!(
        matches!(outcomes.as_slice(), [Outcome::Failed { cause: _ }]),
        "{outcomes:?}"
    );
    assert!(PLACES.iter().all(|place| !Path::new(place).exists()));
    Ok(())
}
