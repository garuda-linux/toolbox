mod background;

use super::{DefaultEntry, GrubKey, GrubSetting, GrubSettings, GrubState, InvalidGrubSettings};
use crate::{
    GrubPicture, GrubThemeFile, GrubTimeout, KernelParameter, KernelParameters, MenuEntryId,
};

/// `/etc/default/grub` of the Garuda Lima image, close to GRUB's stock file.
const LIMA: &str = include_str!("../fixtures/lima/default-grub");
/// `/etc/default/grub` of a used Garuda Mokka install.
const INSTALLED: &str = include_str!("../fixtures/installed/default-grub");
const LIMA_THEME: &str = "/usr/share/grub/themes/garuda/theme.txt";
const INSTALLED_THEME: &str = "/usr/share/grub/themes/catppuccin-mocha/theme.txt";

fn parameters(words: &str) -> KernelParameters {
    KernelParameters::parse(words).unwrap()
}

fn timeout(seconds: i64) -> GrubSetting {
    GrubSetting::Timeout(GrubTimeout::new(seconds).unwrap())
}

fn entry(id: &str) -> GrubSetting {
    GrubSetting::Default(DefaultEntry::Entry(MenuEntryId::new(id).unwrap()))
}

fn changed_lines(before: &str, after: &str) -> Vec<String> {
    assert_eq!(before.lines().count(), after.lines().count());

    before
        .lines()
        .zip(after.lines())
        .filter(|(old, new)| old != new)
        .map(|(_, new)| new.to_owned())
        .collect()
}

#[test]
fn the_lima_file_reads_with_its_theme_and_os_prober() {
    assert_eq!(
        GrubState::read(LIMA),
        GrubState {
            default: "0".to_owned(),
            timeout: Some(5),
            command_line: "loglevel=3 quiet".to_owned(),
            parameters: Some(parameters("loglevel=3 quiet")),
            submenus: true,
            save_default: false,
            os_prober: true,
            background: String::new(),
            menu_picture: String::new(),
            theme: LIMA_THEME.to_owned(),
        }
    );
}

#[test]
fn the_installed_file_reads_with_its_single_quoted_command_line() {
    let state = GrubState::read(INSTALLED);

    assert_eq!(state.default, "0");
    assert_eq!(state.theme, INSTALLED_THEME);
    assert_eq!(state.command_line, "quiet loglevel=3");
    assert!(state.os_prober);
    assert!(state.submenus);
    assert!(state.parameters.as_ref().unwrap().contains("quiet"));
    assert!(!state.parameters.as_ref().unwrap().contains("splash"));
}

#[test]
fn an_empty_file_reads_as_grubs_defaults() {
    let state = GrubState::read("");

    assert_eq!(state.default, "0");
    assert_eq!(state.timeout, Some(5));
    assert_eq!(state.parameters, Some(KernelParameters::default()));
    assert!(state.submenus && !state.save_default && !state.os_prober);
}

#[test]
fn every_spelling_of_no_submenus_counts() {
    for value in ["y", "yes", "true", "\"y\""] {
        assert!(!GrubState::read(&format!("GRUB_DISABLE_SUBMENU={value}\n")).submenus);
    }

    for value in ["n", "false", "\"\""] {
        assert!(GrubState::read(&format!("GRUB_DISABLE_SUBMENU={value}\n")).submenus);
    }
}

/// The page leaves a command line alone that it could not write back
/// word for word.
#[test]
fn a_command_line_with_quotes_is_not_taken_apart() {
    let state = GrubState::read("GRUB_CMDLINE_LINUX_DEFAULT='quiet acpi_osi=\"Windows 2015\"'\n");

    assert_eq!(state.command_line, "quiet acpi_osi=\"Windows 2015\"");
    assert_eq!(state.parameters, None);
    assert_eq!(GrubState::read("GRUB_TIMEOUT=soon\n").timeout, None);
    assert_eq!(GrubState::read("GRUB_TIMEOUT=-1\n").timeout, Some(-1));
}

#[test]
fn settings_write_their_variables_and_values() {
    let cases = [
        (
            GrubSetting::Default(DefaultEntry::Saved),
            "GRUB_DEFAULT",
            Some("saved"),
        ),
        (entry("a>b"), "GRUB_DEFAULT", Some("a>b")),
        (timeout(0), "GRUB_TIMEOUT", Some("0")),
        (
            GrubSetting::KernelParameters(parameters("quiet  splash")),
            "GRUB_CMDLINE_LINUX_DEFAULT",
            Some("quiet splash"),
        ),
        (
            GrubSetting::Submenus(false),
            "GRUB_DISABLE_SUBMENU",
            Some("y"),
        ),
        (
            GrubSetting::Submenus(true),
            "GRUB_DISABLE_SUBMENU",
            Some("n"),
        ),
        (
            GrubSetting::SaveDefault(true),
            "GRUB_SAVEDEFAULT",
            Some("true"),
        ),
        (
            GrubSetting::SaveDefault(false),
            "GRUB_SAVEDEFAULT",
            Some("false"),
        ),
        (
            GrubSetting::OsProber(true),
            "GRUB_DISABLE_OS_PROBER",
            Some("false"),
        ),
        (
            GrubSetting::OsProber(false),
            "GRUB_DISABLE_OS_PROBER",
            Some("true"),
        ),
        (GrubSetting::Background(None), "GRUB_BACKGROUND", None),
        (
            GrubSetting::Theme(Some(GrubThemeFile::new("/t/theme.txt").unwrap())),
            "GRUB_THEME",
            Some("/t/theme.txt"),
        ),
    ];

    for (setting, variable, value) in cases {
        assert_eq!(setting.key().variable(), variable);
        assert_eq!(setting.value().as_deref(), value, "{setting:?}");
    }
}

#[test]
fn a_setting_changes_its_line_and_is_met_afterwards() {
    let settings = [
        entry("gnulinux-advanced-1>gnulinux-linux-lts-1"),
        timeout(12),
        GrubSetting::KernelParameters(parameters("quiet splash mitigations=off")),
        GrubSetting::Submenus(false),
        GrubSetting::SaveDefault(true),
        GrubSetting::OsProber(false),
        GrubSetting::Background(Some(GrubPicture::new("/usr/share/my-wall.png").unwrap())),
        GrubSetting::Theme(Some(
            GrubThemeFile::new("/usr/share/grub/themes/x/theme.txt").unwrap(),
        )),
    ];

    for setting in settings {
        assert!(!setting.is_met_by(&GrubState::read(LIMA)), "{setting:?}");

        let after = setting.apply(LIMA);

        assert_eq!(changed_lines(LIMA, &after).len(), 1, "{setting:?}");
        assert!(setting.is_met_by(&GrubState::read(&after)), "{setting:?}");
        assert_eq!(setting.apply(&after), after);
    }
}

#[test]
fn what_the_file_or_grubs_default_already_says_is_met() {
    let state = GrubState::read(LIMA);

    for setting in [
        entry("0"),
        timeout(5),
        GrubSetting::KernelParameters(parameters("loglevel=3 quiet")),
        GrubSetting::Submenus(true),
        GrubSetting::SaveDefault(false),
        GrubSetting::OsProber(true),
        GrubSetting::Background(None),
        GrubSetting::Theme(Some(GrubThemeFile::new(LIMA_THEME).unwrap())),
    ] {
        assert!(setting.is_met_by(&state), "{setting:?}");
    }
}

#[test]
fn clearing_the_theme_comments_it_out() {
    let after = GrubSetting::Theme(None).apply(INSTALLED);

    assert_eq!(
        changed_lines(INSTALLED, &after),
        [format!("#GRUB_THEME=\"{INSTALLED_THEME}\"")]
    );
    assert_eq!(GrubState::read(&after).theme, "");
}

#[test]
fn several_settings_are_one_edit_with_the_rest_untouched() {
    let settings = GrubSettings::new(vec![
        GrubSetting::Default(DefaultEntry::Saved),
        GrubSetting::SaveDefault(true),
        timeout(3),
    ])
    .unwrap();

    let after = settings.apply(INSTALLED);

    assert_eq!(
        changed_lines(INSTALLED, &after),
        [
            "GRUB_DEFAULT=saved",
            "GRUB_TIMEOUT=3",
            "GRUB_SAVEDEFAULT=true"
        ]
    );
    assert_eq!(
        settings.to_string(),
        "GRUB_DEFAULT=\"saved\", GRUB_SAVEDEFAULT=\"true\", GRUB_TIMEOUT=\"3\""
    );
}

#[test]
fn a_list_must_set_something_and_no_variable_twice() {
    assert_eq!(
        GrubSettings::new(Vec::new()).unwrap_err(),
        InvalidGrubSettings::Empty
    );
    assert_eq!(
        GrubSettings::new(vec![timeout(1), timeout(2)]).unwrap_err(),
        InvalidGrubSettings::Repeated("GRUB_TIMEOUT")
    );
    serde_json::from_str::<GrubSettings>("[]").unwrap_err();
    serde_json::from_str::<GrubSettings>(
        r#"[{"key":"timeout","value":1},{"key":"timeout","value":2}]"#,
    )
    .unwrap_err();
}

#[test]
fn settings_round_trip_through_json_and_check_their_values() {
    let setting = GrubSetting::KernelParameters(parameters("quiet splash"));
    let json = serde_json::to_string(&setting).unwrap();

    assert_eq!(
        json,
        r#"{"key":"kernel-parameters","value":["quiet","splash"]}"#
    );
    assert_eq!(serde_json::from_str::<GrubSetting>(&json).unwrap(), setting);
    assert_eq!(
        serde_json::to_string(&GrubSetting::Default(DefaultEntry::Saved)).unwrap(),
        r#"{"key":"default","value":"saved"}"#
    );
    assert_eq!(
        serde_json::to_string(&entry("a>b")).unwrap(),
        r#"{"key":"default","value":{"entry":"a>b"}}"#
    );
    assert_eq!(
        serde_json::to_string(&GrubSetting::Background(None)).unwrap(),
        r#"{"key":"background","value":null}"#
    );

    for hostile in [
        r#"{"key":"timeout","value":301}"#,
        r#"{"key":"timeout","value":-1}"#,
        r#"{"key":"kernel-parameters","value":["quiet\" ; reboot ; \""]}"#,
        r#"{"key":"kernel-parameters","value":["a b"]}"#,
        r#"{"key":"default","value":{"entry":"$(reboot)"}}"#,
        r#"{"key":"background","value":"/etc/../a.png"}"#,
        r#"{"key":"theme","value":"/a\nGRUB_DEFAULT=x/theme.txt"}"#,
        r#"{"key":"GRUB_PRELOAD_MODULES","value":"x"}"#,
    ] {
        serde_json::from_str::<GrubSetting>(hostile).unwrap_err();
    }
}

#[test]
fn the_parameter_list_adds_once_removes_all_and_is_bounded() {
    let quiet = parameters("quiet splash quiet");
    let splash = KernelParameter::new("splash").unwrap();

    assert_eq!(quiet.with(splash).unwrap(), quiet);
    assert_eq!(quiet.without("quiet").to_string(), "splash");
    assert_eq!(
        quiet
            .with(KernelParameter::new("nowatchdog").unwrap())
            .unwrap()
            .to_string(),
        "quiet splash quiet nowatchdog"
    );

    let many: Vec<String> = (0_u32..129_u32)
        .map(|number| format!("p{number}"))
        .collect();

    assert_eq!(KernelParameters::parse(&many.join(" ")), None);
    assert!(KernelParameters::parse(&many[..128].join(" ")).is_some());
    assert_eq!(KernelParameters::parse("a \"b c\""), None);
}

#[test]
fn the_parameter_list_holds_at_most_1024_bytes_joined() {
    let fits = [255, 255, 255, 254]
        .map(|length| "a".repeat(length))
        .join(" ")
        + " b";

    assert_eq!(fits.len(), 1024);
    assert_eq!(parameters(&fits).to_string(), fits);
    assert_eq!(KernelParameters::parse(&format!("{fits}c")), None);
    parameters(&fits)
        .with(KernelParameter::new("d").unwrap())
        .unwrap_err();
}

#[test]
fn keys_have_stable_ids() {
    let all = [
        GrubKey::Default,
        GrubKey::Timeout,
        GrubKey::KernelParameters,
        GrubKey::Submenus,
        GrubKey::SaveDefault,
        GrubKey::OsProber,
        GrubKey::Background,
        GrubKey::Theme,
    ];

    assert_eq!(
        all.map(GrubKey::as_str),
        [
            "default",
            "timeout",
            "kernel-parameters",
            "submenus",
            "save-default",
            "os-prober",
            "background",
            "theme"
        ]
    );

    for key in all {
        assert_eq!(
            serde_json::to_string(&key).unwrap(),
            format!("\"{}\"", key.as_str())
        );
    }
}

#[test]
fn the_review_names_an_unset_variable_with_a_hash() {
    let settings = GrubSettings::new(vec![GrubSetting::Theme(None), timeout(3)]).unwrap();

    assert_eq!(settings.to_string(), "#GRUB_THEME, GRUB_TIMEOUT=\"3\"");
}
