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

/// The Home card listed the self-test's copies while the snapshot asked the
/// real `pacdiff`, which found none, so a queued merge counted as done and
/// left the queue at once.
#[cfg(feature = "selftest")]
#[test]
fn the_snapshot_finds_the_copies_the_home_card_shows() -> std::io::Result<()> {
    let configs = tempfile::tempdir()?;
    std::fs::create_dir_all(configs.path().join("root/etc"))?;
    std::fs::write(configs.path().join("root/etc/locale.gen.pacsave"), "")?;

    let folder = configs.path().as_os_str().to_owned();
    let lookup = |key: &str| (key == "GARUDA_TOOLBOX_SELFTEST_CONFIGS").then(|| folder.clone());

    assert_eq!(
        super::snapshot_sources_with(lookup, true)
            .merges
            .waiting()
            .ok(),
        Some(vec![PathBuf::from("/etc/locale.gen.pacsave")])
    );

    Ok(())
}
