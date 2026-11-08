use std::fs;
use std::time::Duration;

use toolbox_plan::SchemeId;

use super::{Language, Loaded, LogLevel, RefreshInterval, Settings};

const INTERVALS: [(RefreshInterval, &str, Option<u64>); 8] = [
    (RefreshInterval::Off, "off", None),
    (RefreshInterval::Minutes(5), "5m", Some(300)),
    (RefreshInterval::Minutes(30), "30m", Some(1_800)),
    (RefreshInterval::Minutes(45), "45m", Some(2_700)),
    (RefreshInterval::Minutes(60), "1h", Some(3_600)),
    (RefreshInterval::Minutes(90), "90m", Some(5_400)),
    (RefreshInterval::Minutes(360), "6h", Some(21_600)),
    (RefreshInterval::Minutes(1440), "24h", Some(86_400)),
];

/// `text` as the settings file, loaded.
fn load(text: &str) -> Loaded {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.toml");
    fs::write(&path, text).unwrap();

    Settings::load(&path)
}

#[test]
fn defaults_follow_the_system() {
    let settings = Settings::default();

    assert_eq!(settings.general.language, None);
    assert_eq!(settings.general.log_level, LogLevel::Info);
    assert!(!settings.diagnostics.copy_to_clipboard);
    assert!(!settings.services.show_user_units);
    assert!(!settings.packages.show_aur);
    assert_eq!(settings.status.refresh, RefreshInterval::Minutes(60));
}

#[test]
fn a_missing_file_gives_defaults_without_a_problem() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = Settings::load(&dir.path().join("settings.toml"));

    assert_eq!(loaded.settings, Settings::default());
    assert!(loaded.problem.is_none());
}

#[test]
fn a_broken_file_gives_defaults_and_says_why() {
    let loaded = load("[general\nlog_level = ");

    assert_eq!(loaded.settings, Settings::default());
    let problem = loaded.problem.unwrap().to_string();
    assert!(problem.contains("settings.toml"), "{problem}");
}

#[test]
fn a_partial_file_keeps_defaults_for_the_rest() {
    let loaded = load("[general]\nlanguage = \"de\"\n");

    assert_eq!(
        loaded.settings.general.language,
        Some(Language::new("de").unwrap())
    );
    assert_eq!(loaded.settings.general.log_level, LogLevel::Info);
}

#[test]
fn a_setting_an_older_version_wrote_is_ignored_without_a_problem() {
    let loaded = load(
        "[general]\nlanguage = \"de\"\n[appearance]\nblur = \"off\"\nshow_main_links = false\n",
    );

    assert!(loaded.problem.is_none(), "{:?}", loaded.problem);
    assert_eq!(
        loaded.settings.general.language,
        Some(Language::new("de").unwrap())
    );

    let written = toml::to_string(&loaded.settings).unwrap();
    assert!(!written.contains("appearance"), "{written}");
    assert!(!written.contains("blur"), "{written}");
}

#[test]
fn saved_settings_load_back_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/settings.toml");
    let mut settings = Settings::default();
    settings.general.language = Some(Language::new("pt_BR").unwrap());
    settings.general.log_level = LogLevel::Debug;
    settings.diagnostics.copy_to_clipboard = true;
    settings.packages.show_aur = true;
    settings.status.refresh = RefreshInterval::Minutes(360);

    settings.save(&path).unwrap();

    let loaded = Settings::load(&path);
    assert!(loaded.problem.is_none());
    assert_eq!(loaded.settings, settings);
}

#[test]
fn an_invalid_language_in_the_file_is_a_problem() {
    let loaded = load("[general]\nlanguage = \"../../etc\"\n");

    assert_eq!(loaded.settings, Settings::default());
    assert!(loaded.problem.is_some());
}

#[test]
fn languages_are_short_locale_codes() {
    for code in ["en", "de", "pt_BR", "zh_CN", "sr-Latn"] {
        Language::new(code).unwrap();
    }

    for code in ["", "e", "../x", "de DE", "a-very-long-language-tag"] {
        Language::new(code).unwrap_err();
    }
}

#[test]
fn refresh_intervals_read_from_their_short_names() {
    for (interval, text, _) in INTERVALS {
        let settings: Settings =
            toml::from_str(&format!("[status]\nrefresh = \"{text}\"\n")).unwrap();

        assert_eq!(settings.status.refresh, interval, "{text}");
    }
}

#[test]
fn refresh_intervals_write_their_short_names() {
    for (interval, text, _) in INTERVALS {
        let mut settings = Settings::default();
        settings.status.refresh = interval;

        let written = toml::to_string_pretty(&settings).unwrap();

        assert!(
            written.contains(&format!("refresh = \"{text}\"")),
            "{written}"
        );
    }
}

#[test]
fn refresh_intervals_give_their_period() {
    for (interval, text, seconds) in INTERVALS {
        assert_eq!(
            interval.period(),
            seconds.map(Duration::from_secs),
            "{text}"
        );
    }
}

#[test]
fn an_unknown_refresh_interval_is_a_problem_like_any_bad_value() {
    let loaded = load("[status]\nrefresh = \"soon\"\n");

    assert_eq!(loaded.settings, Settings::default());
    assert!(loaded.problem.is_some());
}

#[test]
fn one_bad_field_keeps_the_valid_ones() {
    let loaded = load("[general]\nlanguage = \"de\"\n[status]\nrefresh = \"soon\"\n");

    assert_eq!(
        loaded.settings.general.language,
        Some(Language::new("de").unwrap())
    );
    assert_eq!(loaded.settings.status.refresh, RefreshInterval::Minutes(60));
    assert!(loaded.problem.is_some());
}

#[test]
fn a_bad_field_does_not_take_its_section_neighbours_with_it() {
    let loaded =
        load("[general]\nlanguage = \"de\"\nlog_level = \"loud\"\n[status]\nrefresh = \"soon\"\n");

    assert_eq!(
        loaded.settings.general.language,
        Some(Language::new("de").unwrap())
    );
    assert_eq!(loaded.settings.general.log_level, LogLevel::Info);
    assert_eq!(loaded.settings.status.refresh, RefreshInterval::Minutes(60));
    assert!(loaded.problem.is_some());
}

#[test]
fn an_unknown_key_is_not_settings_and_does_not_disturb_them() {
    let loaded = load("[general]\nlanguage = \"de\"\nsomething_new = 1\n[elsewhere]\nx = 2\n");

    assert_eq!(loaded.settings.general.log_level, LogLevel::Info);
    assert_eq!(
        loaded.settings.general.language,
        Some(Language::new("de").unwrap())
    );
}

#[test]
fn every_log_level_maps_to_a_tracing_filter() {
    use tracing::level_filters::LevelFilter;

    assert_eq!(LevelFilter::from(LogLevel::Error), LevelFilter::ERROR);
    assert_eq!(LevelFilter::from(LogLevel::Warn), LevelFilter::WARN);
    assert_eq!(LevelFilter::from(LogLevel::Info), LevelFilter::INFO);
    assert_eq!(LevelFilter::from(LogLevel::Debug), LevelFilter::DEBUG);
    assert_eq!(LevelFilter::from(LogLevel::Trace), LevelFilter::TRACE);
}

#[test]
fn a_language_prints_its_code() {
    assert_eq!(Language::new("pt_BR").unwrap().to_string(), "pt_BR");
}

#[test]
fn an_unreadable_file_is_a_problem_not_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    fs::write(&blocker, "").unwrap();
    let loaded = Settings::load(&blocker.join("settings.toml"));

    assert_eq!(loaded.settings, Settings::default());
    assert!(
        loaded
            .problem
            .unwrap()
            .to_string()
            .contains("settings.toml")
    );
}

#[test]
fn an_unwritable_file_fails_saving() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    fs::write(&blocker, "").unwrap();

    Settings::default()
        .save(&blocker.join("settings.toml"))
        .unwrap_err();
}

#[test]
fn a_refresh_outside_five_minutes_to_a_day_is_a_problem() {
    for text in ["4m", "0m", "1441m", "25h", "-5m", "m"] {
        let read: Result<Settings, _> =
            toml::from_str(&format!("[status]\nrefresh = \"{text}\"\n"));

        assert!(read.is_err(), "{text}");
    }
}

#[test]
fn hours_read_as_their_minutes() {
    let settings: Settings = toml::from_str("[status]\nrefresh = \"3h\"\n").unwrap();

    assert_eq!(settings.status.refresh, RefreshInterval::Minutes(180));
}

#[test]
fn the_aur_shows_only_once_turned_on() {
    let loaded = load("[packages]\nshow_aur = true\n");

    assert!(loaded.problem.is_none());
    assert!(loaded.settings.packages.show_aur);
    assert!(!load("").settings.packages.show_aur);
}

#[test]
fn the_colour_schemes_follow_the_desktop_until_one_is_picked() {
    let settings = Settings::default();
    assert_eq!(settings.colours.dark_scheme, None);
    assert_eq!(settings.colours.light_scheme, None);

    let loaded = load("[colours]\ndark_scheme = \"nord\"\nlight_scheme = \"catppuccin-latte\"\n");
    assert!(loaded.problem.is_none(), "{:?}", loaded.problem);
    assert_eq!(
        loaded
            .settings
            .colours
            .dark_scheme
            .as_ref()
            .map(SchemeId::as_str),
        Some("nord")
    );
    assert_eq!(
        loaded
            .settings
            .colours
            .light_scheme
            .as_ref()
            .map(SchemeId::as_str),
        Some("catppuccin-latte")
    );
}

#[test]
fn a_bad_scheme_id_follows_the_desktop_and_keeps_the_rest() {
    let loaded = load("[colours]\ndark_scheme = \"Nord!\"\nlight_scheme = \"nord-light\"\n");

    assert!(loaded.problem.is_some());
    assert_eq!(loaded.settings.colours.dark_scheme, None);
    assert_eq!(
        loaded.settings.colours.light_scheme,
        Some(SchemeId::new("nord-light").unwrap())
    );
}
