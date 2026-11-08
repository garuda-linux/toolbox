use super::{
    ConfigWord, EditError, HomePath, InvalidConfigWord, InvalidHomePath, Rewrite, UserConfigEdit,
};

fn path(value: &str) -> HomePath {
    HomePath::new(value).unwrap()
}

fn word(value: &str) -> ConfigWord {
    ConfigWord::new(value).unwrap()
}

fn file() -> UserConfigEdit {
    UserConfigEdit::File {
        path: path(".config/wireplumber/wireplumber.conf.d/disable-suspension.conf"),
        content: "monitor.alsa.rules = []\n".to_owned(),
    }
}

fn kwin() -> UserConfigEdit {
    UserConfigEdit::IniKey {
        path: path(".config/kwinrc"),
        group: word("Windows"),
        key: word("BorderlessMaximizedWindows"),
        on: word("true"),
        off: word("false"),
    }
}

fn discord() -> UserConfigEdit {
    UserConfigEdit::JsonFlag {
        path: path(".config/discord/settings.json"),
        key: word("SKIP_HOST_UPDATE"),
    }
}

#[test]
fn home_paths_stay_below_the_home() {
    assert_eq!(path(".config/kwinrc").as_str(), ".config/kwinrc");
    assert_eq!(path(".config/kwinrc").to_string(), "~/.config/kwinrc");
    assert_eq!(HomePath::new(""), Err(InvalidHomePath::Empty));
    assert!(matches!(
        HomePath::new("/etc/passwd"),
        Err(InvalidHomePath::Absolute(_))
    ));

    for bad in [
        "../x",
        ".config/../../x",
        "./x",
        ".config//x",
        ".config/",
        "..",
    ] {
        assert!(
            matches!(
                HomePath::new(bad),
                Err(InvalidHomePath::InvalidComponent(_))
            ),
            "{bad}"
        );
    }

    assert_eq!(
        HomePath::new("a\nb"),
        Err(InvalidHomePath::InvalidCharacter('\n'))
    );
    assert!(matches!(
        HomePath::new("a".repeat(2000)),
        Err(InvalidHomePath::TooLong { max: _ })
    ));
}

#[test]
fn config_words_cannot_change_the_structure_of_a_file() {
    assert_eq!(word("SKIP_HOST_UPDATE").as_str(), "SKIP_HOST_UPDATE");
    assert_eq!(ConfigWord::new(""), Err(InvalidConfigWord::Empty));

    for (bad, character) in [
        ("a=b", '='),
        ("[x", '['),
        ("x]", ']'),
        ("#x", '#'),
        ("a\nb", '\n'),
    ] {
        assert_eq!(
            ConfigWord::new(bad),
            Err(InvalidConfigWord::InvalidCharacter(character))
        );
    }

    assert!(matches!(
        ConfigWord::new(" x"),
        Err(InvalidConfigWord::Padded(_))
    ));
}

#[test]
fn a_file_is_on_when_it_exists() {
    assert!(file().is_on(Some(b"anything")));
    assert!(!file().is_on(None));
}

#[test]
fn a_file_is_written_or_removed() {
    let content = b"monitor.alsa.rules = []\n".as_slice();

    assert_eq!(
        file().rewrite(None, true),
        Ok(Rewrite::Write("monitor.alsa.rules = []\n".to_owned()))
    );
    assert_eq!(file().rewrite(Some(content), true), Ok(Rewrite::Unchanged));
    assert_eq!(file().rewrite(Some(b"old"), false), Ok(Rewrite::Remove));
    assert_eq!(file().rewrite(None, false), Ok(Rewrite::Unchanged));
}

#[test]
fn an_ini_key_is_on_with_its_on_value() {
    assert!(kwin().is_on(Some(b"[Windows]\nBorderlessMaximizedWindows=true\n")));
    assert!(!kwin().is_on(Some(b"[Windows]\nBorderlessMaximizedWindows=false\n")));
    assert!(!kwin().is_on(Some(b"[Other]\nBorderlessMaximizedWindows=true\n")));
    assert!(!kwin().is_on(None));
}

/// A key the file lacks is added, not skipped.
#[test]
fn turning_an_ini_key_on_creates_it_in_a_fresh_file() {
    assert_eq!(
        kwin().rewrite(None, true),
        Ok(Rewrite::Write(
            "[Windows]\nBorderlessMaximizedWindows=true\n".to_owned()
        ))
    );
    assert_eq!(
        kwin().rewrite(Some(b"[Plugins]\nblurEnabled=true\n"), true),
        Ok(Rewrite::Write(
            "[Plugins]\nblurEnabled=true\n\n[Windows]\nBorderlessMaximizedWindows=true\n"
                .to_owned()
        ))
    );
}

#[test]
fn turning_an_ini_key_off_only_touches_an_existing_entry() {
    assert_eq!(kwin().rewrite(None, false), Ok(Rewrite::Unchanged));
    assert_eq!(
        kwin().rewrite(Some(b"[Windows]\nFocusPolicy=Click\n"), false),
        Ok(Rewrite::Unchanged)
    );
    assert_eq!(
        kwin().rewrite(Some(b"[Windows]\nBorderlessMaximizedWindows=true\n"), false),
        Ok(Rewrite::Write(
            "[Windows]\nBorderlessMaximizedWindows=false\n".to_owned()
        ))
    );
}

#[test]
fn files_that_are_no_text_are_refused() {
    assert_eq!(
        kwin().rewrite(Some(&[0xff, 0xfe]), true),
        Err(EditError::NotText)
    );
    assert_eq!(
        discord().rewrite(Some(&[0xff, 0xfe]), true),
        Err(EditError::NotText)
    );
    assert!(!kwin().is_on(Some(&[0xff, 0xfe])));
}

#[test]
fn a_json_flag_keeps_the_other_settings() {
    assert!(discord().is_on(Some(b"{\"SKIP_HOST_UPDATE\": true}")));
    assert_eq!(
        discord().rewrite(Some(b"[]"), true),
        Err(EditError::NotJsonObject)
    );
    assert_eq!(
        discord().rewrite(None, true),
        Ok(Rewrite::Write(
            "{\n  \"SKIP_HOST_UPDATE\": true\n}\n".to_owned()
        ))
    );
}

#[test]
fn names_and_descriptions_say_which_file_and_key() {
    assert_eq!(
        file().to_string(),
        "~/.config/wireplumber/wireplumber.conf.d/disable-suspension.conf"
    );
    assert_eq!(
        kwin().to_string(),
        "~/.config/kwinrc [Windows] BorderlessMaximizedWindows"
    );
    assert_eq!(
        discord().to_string(),
        "~/.config/discord/settings.json SKIP_HOST_UPDATE"
    );
    assert_eq!(
        kwin().describe(true),
        "set BorderlessMaximizedWindows=true in [Windows] of ~/.config/kwinrc"
    );
    assert_eq!(
        discord().describe(false),
        "remove \"SKIP_HOST_UPDATE\" from ~/.config/discord/settings.json"
    );
    assert_eq!(file().describe(false), format!("rm -f {}", file().path()));
}

#[test]
fn edits_survive_json_and_refuse_bad_paths() {
    for edit in [file(), kwin(), discord()] {
        let json = serde_json::to_string(&edit).unwrap();

        assert_eq!(serde_json::from_str::<UserConfigEdit>(&json).unwrap(), edit);
    }

    let bad = r#"{"kind":"json-flag","path":"../.ssh/config","key":"x"}"#;

    serde_json::from_str::<UserConfigEdit>(bad).unwrap_err();
}

#[test]
fn paths_and_words_parse_from_str_and_borrow_as_str() {
    let parsed: HomePath = ".config/kwinrc".parse().unwrap();

    assert_eq!(parsed.as_ref(), ".config/kwinrc");
    assert_eq!(parsed.to_string(), "~/.config/kwinrc");
    "../x".parse::<HomePath>().unwrap_err();

    let word: ConfigWord = "true".parse().unwrap();

    assert_eq!(word.as_ref(), "true");
    assert_eq!(word.to_string(), "true");
    "a=b".parse::<ConfigWord>().unwrap_err();
}

#[test]
fn turning_on_names_the_file_and_the_value() {
    assert_eq!(file().describe(true), format!("write {}", file().path()));
    assert_eq!(
        discord().describe(true),
        "set \"SKIP_HOST_UPDATE\": true in ~/.config/discord/settings.json"
    );
}
