use std::ffi::OsString;
use std::path::PathBuf;

use super::self_test_configs;

fn variable(key: &str) -> Option<OsString> {
    (key == "GARUDA_TOOLBOX_SELFTEST_CONFIGS").then(|| OsString::from("/srv/configs"))
}

#[test]
fn a_development_build_takes_the_self_test_configs() {
    assert_eq!(
        self_test_configs(variable, true),
        Some(PathBuf::from("/srv/configs"))
    );
}

#[test]
fn a_release_ignores_the_self_test_configs() {
    assert_eq!(self_test_configs(variable, false), None);
}
