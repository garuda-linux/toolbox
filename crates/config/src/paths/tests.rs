use std::ffi::OsString;
use std::path::PathBuf;

use super::Paths;

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();

    move |key| {
        pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| OsString::from(value))
    }
}

#[test]
fn the_home_is_the_named_one_else_root() {
    let named = Paths::from_env(env(&[("HOME", "/home/nico")]));
    let unnamed = Paths::from_env(env(&[]));

    assert_eq!(named.home(), PathBuf::from("/home/nico"));
    assert_eq!(unnamed.home(), PathBuf::from("/root"));
}

#[test]
fn xdg_variables_win_over_home() {
    let paths = Paths::from_env(env(&[
        ("HOME", "/home/nico"),
        ("XDG_CONFIG_HOME", "/cfg"),
        ("XDG_STATE_HOME", "/state"),
        ("XDG_CACHE_HOME", "/cache"),
    ]));

    assert_eq!(
        paths.settings_file(),
        PathBuf::from("/cfg/garuda-toolbox/settings.toml")
    );
    assert_eq!(paths.state_dir(), PathBuf::from("/state/garuda-toolbox"));
    assert_eq!(paths.cache_dir(), PathBuf::from("/cache/garuda-toolbox"));
    assert_eq!(paths.aur_dir(), PathBuf::from("/cache/garuda-toolbox/aur"));
    assert_eq!(paths.autostart_dir(), PathBuf::from("/cfg/autostart"));
}

#[test]
fn home_gives_the_xdg_defaults() {
    let paths = Paths::from_env(env(&[("HOME", "/home/nico")]));

    assert_eq!(
        paths.settings_file(),
        PathBuf::from("/home/nico/.config/garuda-toolbox/settings.toml")
    );
    assert_eq!(
        paths.state_dir(),
        PathBuf::from("/home/nico/.local/state/garuda-toolbox")
    );
    assert_eq!(
        paths.cache_dir(),
        PathBuf::from("/home/nico/.cache/garuda-toolbox")
    );
}

#[test]
fn relative_xdg_values_are_ignored_as_the_spec_says() {
    let paths = Paths::from_env(env(&[("HOME", "/home/nico"), ("XDG_CONFIG_HOME", "cfg")]));

    assert_eq!(
        paths.autostart_dir(),
        PathBuf::from("/home/nico/.config/autostart")
    );
}

#[test]
fn state_files_live_below_the_state_dir() {
    let paths = Paths::from_env(env(&[("HOME", "/home/nico")]));

    assert_eq!(
        paths.staged_changes_file(),
        PathBuf::from("/home/nico/.local/state/garuda-toolbox/changes.json")
    );
    assert_eq!(
        paths.crash_file(),
        PathBuf::from("/home/nico/.local/state/garuda-toolbox/crash.log")
    );
}

#[test]
fn relative_folders_and_a_missing_home_fall_back() {
    let paths = Paths::from_env(env(&[
        ("HOME", "/home/nico"),
        ("XDG_STATE_HOME", "relative"),
    ]));

    assert_eq!(
        paths.state_dir(),
        PathBuf::from("/home/nico/.local/state/garuda-toolbox")
    );

    let rootless = Paths::from_env(env(&[]));

    assert_eq!(
        rootless.state_dir(),
        PathBuf::from("/root/.local/state/garuda-toolbox")
    );
}

#[test]
fn the_process_paths_name_the_app() {
    let paths = Paths::from_process();

    assert!(
        paths
            .settings_file()
            .ends_with("garuda-toolbox/settings.toml")
    );
    assert!(
        paths
            .staged_changes_file()
            .ends_with("garuda-toolbox/changes.json")
    );
    assert!(paths.crash_file().ends_with("garuda-toolbox/crash.log"));
}

#[test]
fn the_own_state_folder_needs_a_home_or_a_state_home() {
    let homed = Paths::from_env(env(&[("HOME", "/home/nico")]));
    let stated = Paths::from_env(env(&[("XDG_STATE_HOME", "/state")]));
    let relative = Paths::from_env(env(&[("XDG_STATE_HOME", "state")]));

    assert_eq!(
        homed.own_state_dir(),
        Some(PathBuf::from("/home/nico/.local/state/garuda-toolbox"))
    );
    assert_eq!(
        stated.own_state_dir(),
        Some(PathBuf::from("/state/garuda-toolbox"))
    );
    assert_eq!(relative.own_state_dir(), None);
    assert_eq!(Paths::from_env(env(&[])).own_state_dir(), None);
}

/// A relative home would put the app's files below whatever folder the
/// app was started in.
#[test]
fn a_relative_home_counts_as_none() {
    let paths = Paths::from_env(env(&[("HOME", "home/nico")]));

    assert_eq!(paths.home(), PathBuf::from("/root"));
    assert_eq!(paths.own_state_dir(), None);
}
