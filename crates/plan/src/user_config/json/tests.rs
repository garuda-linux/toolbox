use super::{is_on, set};
use crate::EditError;

const KEY: &str = "SKIP_HOST_UPDATE";

/// `~/.config/discord/settings.json` of a used install, no flag in it and
/// no line break at its end.
const DISCORD: &str = crate::testing::user_config::INSTALLED_DISCORD_SETTINGS_JSON;

/// Edit: the real file with `"SKIP_HOST_UPDATE": <value>` as its first key.
fn with_flag(value: &str) -> String {
    DISCORD.replacen("{\n", &format!("{{\n  \"{KEY}\": {value},\n"), 1)
}

/// Edit: the real file cut in half.
fn cut() -> &'static str {
    DISCORD.get(..DISCORD.len() / 2).unwrap()
}

#[test]
fn on_means_the_key_is_true() {
    assert!(!is_on(DISCORD, KEY));
    assert!(is_on(&with_flag("true"), KEY));
    assert!(!is_on(&with_flag("false"), KEY));
    assert!(!is_on(&with_flag("\"true\""), KEY));
    assert!(!is_on("", KEY));
    assert!(!is_on(cut(), KEY));
    assert!(!is_on("[true]", KEY));
}

/// Discord writes two blanks of indent like `to_string_pretty`, so the
/// edit adds one line and the final line break.
#[test]
fn turning_on_keeps_the_other_keys_in_their_order() {
    let after = set(DISCORD, KEY, true).unwrap().unwrap();

    assert_eq!(
        after,
        format!(
            "{},\n  \"{KEY}\": true\n}}\n",
            DISCORD.strip_suffix("\n}").unwrap()
        )
    );
}

#[test]
fn turning_off_removes_only_the_key() {
    let after = set(&with_flag("true"), KEY, false).unwrap().unwrap();

    assert_eq!(after, format!("{DISCORD}\n"));
}

#[test]
fn a_missing_or_blank_file_is_an_empty_object() {
    assert_eq!(
        set("", KEY, true).unwrap().unwrap(),
        "{\n  \"SKIP_HOST_UPDATE\": true\n}\n"
    );
    assert_eq!(set(" \n", KEY, false).unwrap(), None);
}

#[test]
fn a_file_already_as_wanted_is_left_alone() {
    assert_eq!(set(&with_flag("true"), KEY, true).unwrap(), None);
    assert_eq!(set(DISCORD, KEY, false).unwrap(), None);
}

#[test]
fn a_false_flag_turns_true_in_its_place() {
    assert_eq!(
        set(&with_flag("false"), KEY, true).unwrap(),
        Some(format!("{}\n", with_flag("true")))
    );
}

#[test]
fn refuses_what_is_no_json_object() {
    assert_eq!(set("[1, 2]", KEY, true), Err(EditError::NotJsonObject));
    assert!(matches!(
        set(cut(), KEY, true),
        Err(EditError::InvalidJson(_))
    ));
}
