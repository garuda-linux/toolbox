use toolbox_catalog::ColourSchemeCatalog;
use toolbox_cli::args::SettingKey;
use toolbox_config::{RefreshInterval, Settings};

use super::{rows, set, value};

#[test]
fn every_setting_reads_back_what_it_was_set_to() {
    let cases = [
        (SettingKey::Language, "de"),
        (SettingKey::UpdateCheck, "30"),
        (SettingKey::UserServices, "on"),
        (SettingKey::CopyReports, "on"),
        (SettingKey::Aur, "on"),
        (SettingKey::LogLevel, "debug"),
        (SettingKey::DarkScheme, "nord"),
        (SettingKey::LightScheme, "catppuccin-latte"),
    ];

    for (key, wanted) in cases {
        let changed = set(Settings::default(), key, wanted, &schemes()).unwrap();
        assert_eq!(value(&changed, key, false), wanted, "{key:?}");
    }
}

#[test]
fn the_system_language_and_no_update_check_have_words() {
    let changed = set(
        Settings::default(),
        SettingKey::Language,
        "system",
        &schemes(),
    )
    .unwrap();

    assert_eq!(changed.general.language, None);

    let never = set(
        Settings::default(),
        SettingKey::UpdateCheck,
        "never",
        &schemes(),
    )
    .unwrap();

    assert_eq!(never.status.refresh, RefreshInterval::Off);
}

#[test]
fn bad_values_say_what_they_should_have_been() {
    set(
        Settings::default(),
        SettingKey::LogLevel,
        "loud",
        &schemes(),
    )
    .unwrap_err();
    set(
        Settings::default(),
        SettingKey::UpdateCheck,
        "soon",
        &schemes(),
    )
    .unwrap_err();
    set(
        Settings::default(),
        SettingKey::Language,
        "Deutsch!",
        &schemes(),
    )
    .unwrap_err();
    set(
        Settings::default(),
        SettingKey::StartAtLogin,
        "on",
        &schemes(),
    )
    .unwrap_err();
}

#[test]
fn the_list_holds_every_setting_with_its_value() {
    let mut settings = Settings::default();
    settings.packages.show_aur = true;

    let listed = rows(&settings, true);
    assert_eq!(listed.len(), 9);
    assert!(
        listed
            .iter()
            .any(|row| row.key == "aur" && row.value == "on")
    );
    assert!(
        listed
            .iter()
            .any(|row| row.key == "start-at-login" && row.value == "on")
    );
}

#[test]
fn one_setting_reads_as_json_like_a_row_of_the_list() {
    use toolbox_cli::args::Read;

    use super::get_text;

    let text = get_text(
        Read { json: true },
        &Settings::default(),
        SettingKey::Language,
        false,
    );

    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let shown = value(&Settings::default(), SettingKey::Language, false);
    assert_eq!(
        parsed,
        serde_json::json!({ "schema": 1_u64, "id": "language", "value": shown })
    );

    let plain = get_text(
        Read { json: false },
        &Settings::default(),
        SettingKey::Language,
        false,
    );

    assert_eq!(plain, format!("{}\n", parsed["value"].as_str().unwrap()));
}

#[test]
fn the_settings_list_has_a_summary_and_a_hint() {
    use super::setting_listing;

    let listed = rows(&Settings::default(), false);
    let listing = setting_listing(&listed);
    assert_eq!(listing.summary.as_deref(), Some("9"));
    assert!(listing.hint.unwrap().contains("settings set"));
}

/// The value is a word of its own, the one `settings set log-level` takes,
/// never the program's name for it.
#[test]
fn every_log_level_reads_as_the_word_that_sets_it() {
    use toolbox_config::LogLevel;

    for level in [
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ] {
        assert_eq!(super::log_level(super::log_level_name(level)), Ok(level));
    }

    assert_eq!(super::log_level_name(LogLevel::Warn), "warn");
}

fn schemes() -> ColourSchemeCatalog {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    ColourSchemeCatalog::load(&dir).unwrap()
}

#[test]
fn system_takes_the_desktops_colours_back() {
    let mut settings = Settings::default();
    settings.colours.dark_scheme = Some(toolbox_plan::SchemeId::new("nord").unwrap());

    let changed = set(settings, SettingKey::DarkScheme, "system", &schemes()).unwrap();
    assert_eq!(changed.colours.dark_scheme, None);
    assert_eq!(value(&changed, SettingKey::DarkScheme, false), "system");
}

#[test]
fn a_scheme_must_be_one_of_the_catalog_for_its_mode() {
    for (key, odd) in [
        (SettingKey::DarkScheme, "solarized"),
        (SettingKey::DarkScheme, "catppuccin-latte"),
        (SettingKey::LightScheme, "nord"),
        (SettingKey::LightScheme, "Nord Light"),
    ] {
        let message = set(Settings::default(), key, odd, &schemes())
            .unwrap_err()
            .to_string();

        assert!(message.contains("system"), "{odd}: {message}");
    }
}

#[test]
fn a_refused_value_reads_as_before_the_error_type() {
    let refused = |key, value| {
        set(Settings::default(), key, value, &schemes())
            .unwrap_err()
            .to_string()
    };

    assert_eq!(
        refused(SettingKey::UpdateCheck, "soon"),
        "`soon` is no number of minutes, 5 to 1440, or never"
    );
    assert_eq!(
        refused(SettingKey::Aur, "maybe"),
        "`maybe` is neither on nor off"
    );
    assert_eq!(
        refused(SettingKey::StartAtLogin, "on"),
        "start at login is no setting of the file"
    );
}
