use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;

use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_plan::{ConfigWord, HomePath, StepId, UserConfigEdit};

use super::edit_user_config;
use crate::builtin::{FileChange, execute};
use crate::testing::{assert_failed, mode};

fn word(value: &str) -> ConfigWord {
    ConfigWord::new(value).unwrap()
}

fn audio() -> UserConfigEdit {
    UserConfigEdit::File {
        path: HomePath::new(".config/wireplumber/wireplumber.conf.d/disable-suspension.conf")
            .unwrap(),
        content: "monitor.alsa.rules = []\n".to_owned(),
    }
}

fn kwin() -> UserConfigEdit {
    UserConfigEdit::IniKey {
        path: HomePath::new(".config/kwinrc").unwrap(),
        group: word("Windows"),
        key: word("BorderlessMaximizedWindows"),
        on: word("true"),
        off: word("false"),
    }
}

fn discord() -> UserConfigEdit {
    UserConfigEdit::JsonFlag {
        path: HomePath::new(".config/discord/settings.json").unwrap(),
        key: word("SKIP_HOST_UPDATE"),
    }
}

fn read(home: &Path, path: &str) -> String {
    fs::read_to_string(home.join(path)).unwrap()
}

#[test]
fn a_file_is_created_with_its_folders_and_removed_again() {
    let home = tempfile::tempdir().unwrap();
    let path = ".config/wireplumber/wireplumber.conf.d/disable-suspension.conf";

    assert_eq!(
        edit_user_config(home.path(), &audio(), true).unwrap(),
        FileChange::Written
    );
    assert_eq!(read(home.path(), path), "monitor.alsa.rules = []\n");
    assert_eq!(
        edit_user_config(home.path(), &audio(), true).unwrap(),
        FileChange::Unchanged
    );
    assert_eq!(
        edit_user_config(home.path(), &audio(), false).unwrap(),
        FileChange::Removed
    );
    assert!(!home.path().join(path).exists());
    assert_eq!(
        edit_user_config(home.path(), &audio(), false).unwrap(),
        FileChange::Unchanged
    );
}

#[test]
fn a_fresh_kwinrc_gets_the_group_and_key() {
    let home = tempfile::tempdir().unwrap();

    edit_user_config(home.path(), &kwin(), true).unwrap();

    assert_eq!(
        read(home.path(), ".config/kwinrc"),
        "[Windows]\nBorderlessMaximizedWindows=true\n"
    );
}

#[test]
fn an_existing_kwinrc_keeps_everything_else_and_its_mode() {
    let home = tempfile::tempdir().unwrap();
    let kwinrc = home.path().join(".config/kwinrc");
    let before = "[Desktops]\nNumber=2\n\n[Windows]\nFocusPolicy=ClickToFocus\n";

    fs::create_dir_all(kwinrc.parent().unwrap()).unwrap();
    fs::write(&kwinrc, before).unwrap();
    fs::set_permissions(&kwinrc, fs::Permissions::from_mode(0o600)).unwrap();

    edit_user_config(home.path(), &kwin(), true).unwrap();

    assert_eq!(
        read(home.path(), ".config/kwinrc"),
        "[Desktops]\nNumber=2\n\n[Windows]\nFocusPolicy=ClickToFocus\nBorderlessMaximizedWindows=true\n"
    );
    assert_eq!(mode(&kwinrc), 0o600);

    edit_user_config(home.path(), &kwin(), false).unwrap();

    assert_eq!(
        read(home.path(), ".config/kwinrc"),
        "[Desktops]\nNumber=2\n\n[Windows]\nFocusPolicy=ClickToFocus\nBorderlessMaximizedWindows=false\n"
    );
}

#[test]
fn discord_keeps_its_other_settings() {
    let home = tempfile::tempdir().unwrap();
    let settings = home.path().join(".config/discord/settings.json");

    fs::create_dir_all(settings.parent().unwrap()).unwrap();
    fs::write(&settings, "{\"BACKGROUND_COLOR\": \"#202225\"}").unwrap();

    edit_user_config(home.path(), &discord(), true).unwrap();

    let value: serde_json::Value =
        serde_json::from_str(&read(home.path(), ".config/discord/settings.json")).unwrap();

    assert_eq!(
        value,
        serde_json::json!({"BACKGROUND_COLOR": "#202225", "SKIP_HOST_UPDATE": true})
    );

    edit_user_config(home.path(), &discord(), false).unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&read(
            home.path(),
            ".config/discord/settings.json"
        ))
        .unwrap(),
        serde_json::json!({"BACKGROUND_COLOR": "#202225"})
    );
}

#[test]
fn invalid_json_is_left_alone_and_fails() {
    let home = tempfile::tempdir().unwrap();
    let settings = home.path().join(".config/discord/settings.json");

    fs::create_dir_all(settings.parent().unwrap()).unwrap();
    fs::write(&settings, "{\"zoom\": ").unwrap();

    let error = edit_user_config(home.path(), &discord(), true).unwrap_err();

    assert!(error.to_string().contains("not valid JSON"), "{error}");
    assert_eq!(
        read(home.path(), ".config/discord/settings.json"),
        "{\"zoom\": "
    );
}

#[test]
fn a_symlinked_file_stays_a_link_and_its_target_changes() {
    let home = tempfile::tempdir().unwrap();
    let dotfiles = home.path().join("dotfiles");
    let kwinrc = home.path().join(".config/kwinrc");

    fs::create_dir_all(&dotfiles).unwrap();
    fs::create_dir_all(kwinrc.parent().unwrap()).unwrap();
    fs::write(dotfiles.join("kwinrc"), "[Windows]\n").unwrap();
    symlink(dotfiles.join("kwinrc"), &kwinrc).unwrap();

    edit_user_config(home.path(), &kwin(), true).unwrap();

    assert!(
        fs::symlink_metadata(&kwinrc)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(dotfiles.join("kwinrc")).unwrap(),
        "[Windows]\nBorderlessMaximizedWindows=true\n"
    );
}

#[test]
fn the_step_names_the_file_and_fails_without_a_home() {
    let home = tempfile::tempdir().unwrap();
    let builtin = Builtin::EditUserConfig {
        edit: kwin(),
        on: true,
    };

    let mut events = Vec::new();

    assert_eq!(
        execute(
            Path::new("/"),
            Some(home.path()),
            StepId::new(1),
            &builtin,
            &mut events
        ),
        Outcome::Succeeded
    );
    assert_eq!(
        events,
        [Event::Output {
            step: StepId::new(1),
            data: "wrote ~/.config/kwinrc\r\n".to_owned()
        }]
    );

    assert_failed(execute(
        Path::new("/"),
        None,
        StepId::new(2),
        &builtin,
        &mut Vec::new(),
    ));
}

#[test]
fn a_file_in_the_way_of_the_target_is_an_error() {
    let home = tempfile::tempdir().unwrap();

    fs::write(home.path().join(".config"), "").unwrap();

    edit_user_config(home.path(), &audio(), true).unwrap_err();
}

#[test]
fn a_folder_is_no_ini_file() {
    let home = tempfile::tempdir().unwrap();

    fs::create_dir_all(home.path().join(".config/kwinrc")).unwrap();

    edit_user_config(home.path(), &kwin(), true).unwrap_err();
}

#[test]
fn a_write_failure_keeps_no_temporary_file() {
    if rustix::process::geteuid().is_root() {
        return;
    }

    let home = tempfile::tempdir().unwrap();

    fs::create_dir_all(home.path().join(".config")).unwrap();
    fs::write(home.path().join(".config/kwinrc"), "[Windows]\n").unwrap();
    fs::set_permissions(
        home.path().join(".config"),
        fs::Permissions::from_mode(0o555),
    )
    .unwrap();

    edit_user_config(home.path(), &kwin(), true).unwrap_err();

    let left: Vec<_> = fs::read_dir(home.path().join(".config"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();

    assert_eq!(left.len(), 1, "no temporary file is left: {left:?}");

    fs::set_permissions(
        home.path().join(".config"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
}
