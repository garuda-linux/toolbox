use toolbox_plan::{
    ConfigReload, ConfigWord, HomePath, InvalidConfigWord, InvalidHomePath, UnitName,
    UserConfigEdit,
};

use super::{SECTION, problem_of};
use crate::toggles::{Check, SETTINGS, ToggleCatalog, ToggleEntry};
use crate::{CatalogError, Problem};

const INI: &str = "{ user-ini = { path = \".config/kwinrc\", group = \"Windows\", key = \"BorderlessMaximizedWindows\", on = \"true\", off = \"false\" } }";

fn entry(id: &str, check: &str, extra: &str) -> String {
    format!(
        "[[section.entry]]\nid = \"{id}\"\ntitle = \"T\"\ndescription = \"D\"\ncheck = {check}\n{extra}"
    )
}

fn with_check(check: &str, extra: &str) -> String {
    format!("{SECTION}{}", entry("tweak", check, extra))
}

fn word(value: &str) -> ConfigWord {
    ConfigWord::new(value).unwrap()
}

#[test]
fn parses_the_three_kinds_of_user_config() {
    let text = format!(
        "{}{}{}",
        with_check(
            "{ user-file = { path = \".config/a.conf\", content = \"x = 1\\n\" } }",
            "restart-user-units = [\"pipewire.service\", \"wireplumber.service\"]\n"
        ),
        entry("kwin", INI, "desktop = [\"KDE\"]\n"),
        entry(
            "discord",
            "{ user-json = { path = \".config/discord/settings.json\", key = \"SKIP_HOST_UPDATE\" } }",
            ""
        ),
    );

    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();
    let file = catalog.entry("tweak").unwrap();
    let kwin = catalog.entry("kwin").unwrap();
    let discord = catalog.entry("discord").unwrap();

    assert_eq!(
        file.check(),
        &Check::UserConfig(UserConfigEdit::File {
            path: HomePath::new(".config/a.conf").unwrap(),
            content: "x = 1\n".to_owned(),
        })
    );
    assert_eq!(
        file.restart_user_units(),
        [
            UnitName::new("pipewire.service").unwrap(),
            UnitName::new("wireplumber.service").unwrap()
        ]
    );
    assert_eq!(
        kwin.check(),
        &Check::UserConfig(UserConfigEdit::IniKey {
            path: HomePath::new(".config/kwinrc").unwrap(),
            group: word("Windows"),
            key: word("BorderlessMaximizedWindows"),
            on: word("true"),
            off: word("false"),
        })
    );
    assert!(kwin.is_for_desktop(&["kde".to_owned()]));
    assert!(!kwin.is_for_desktop(&["GNOME".to_owned()]));
    assert_eq!(
        discord.check(),
        &Check::UserConfig(UserConfigEdit::JsonFlag {
            path: HomePath::new(".config/discord/settings.json").unwrap(),
            key: word("SKIP_HOST_UPDATE"),
        })
    );
    assert_eq!(discord.restart_user_units(), []);
    assert!(discord.is_for_desktop(&[]));
}

#[test]
fn a_desktop_list_limits_where_the_entry_shows() {
    let catalog =
        ToggleCatalog::parse(&with_check(INI, "desktop = [\"KDE\"]\n"), "test.toml").unwrap();

    let entry = catalog.entry("tweak").unwrap();

    assert!(entry.is_for_desktop(&["kde".to_owned()]));
    assert!(entry.is_for_desktop(&["ubuntu".to_owned(), "KDE".to_owned()]));
    assert!(!entry.is_for_desktop(&["GNOME".to_owned()]));
    assert!(!entry.is_for_desktop(&[]));

    let any = ToggleCatalog::parse(&with_check(INI, ""), "test.toml").unwrap();

    assert!(any.entry("tweak").unwrap().is_for_desktop(&[]));
}

#[test]
fn bad_user_configs_name_file_and_entry() {
    let cases = [
        (
            with_check(
                "{ user-json = { path = \"../.ssh/config\", key = \"x\" } }",
                "",
            ),
            Problem::BadHomePath(InvalidHomePath::InvalidComponent(
                "../.ssh/config".to_owned(),
            )),
        ),
        (
            with_check(
                "{ user-json = { path = \"/etc/passwd\", key = \"x\" } }",
                "",
            ),
            Problem::BadHomePath(InvalidHomePath::Absolute("/etc/passwd".to_owned())),
        ),
        (
            with_check(&INI.replace("\"Windows\"", "\"Win]dows\""), ""),
            Problem::BadConfigWord(InvalidConfigWord::InvalidCharacter(']')),
        ),
        (
            with_check(
                "{ user-file = { path = \".config/a\", content = \" \" } }",
                "",
            ),
            Problem::EmptyContent,
        ),
        (
            with_check(
                "{ package = \"a\" }",
                "restart-user-units = [\"pipewire.service\"]\n",
            ),
            Problem::RestartWithoutUserConfig,
        ),
        (with_check(INI, "desktop = []\n"), Problem::EmptyDesktop),
        (
            with_check(INI, "desktop = [\" \"]\n"),
            Problem::EmptyDesktop,
        ),
    ];

    for (text, expected) in cases {
        assert_eq!(
            problem_of(&text),
            ("test.toml".to_owned(), "tweak".to_owned(), expected)
        );
    }
}

#[test]
fn unknown_fields_of_a_user_config_are_syntax_errors() {
    let text = with_check(
        "{ user-json = { path = \".config/a.json\", key = \"x\", value = 1 } }",
        "",
    );

    assert!(matches!(
        ToggleCatalog::parse(&text, "test.toml"),
        Err(CatalogError::Syntax {
            source_name: _,
            error: _
        })
    ));
}

#[test]
fn the_shipped_settings_catalog_has_the_desktop_and_apps_section() {
    let catalog = ToggleCatalog::load(&crate::source_dir(), SETTINGS).unwrap();
    let section = catalog
        .sections()
        .iter()
        .find(|section| section.id() == "desktop-and-apps")
        .unwrap();

    let ids: Vec<&str> = section.entries().iter().map(ToggleEntry::id).collect();

    assert_eq!(
        ids,
        [
            "audio-no-suspend",
            "kde-borderless-maximize",
            "discord-disable-autoupdate"
        ]
    );
    assert!(
        catalog
            .entry("kde-borderless-maximize")
            .unwrap()
            .is_for_desktop(&["KDE".to_owned()])
    );
    assert_eq!(
        catalog
            .entry("audio-no-suspend")
            .unwrap()
            .restart_user_units()
            .len(),
        3
    );
}

#[test]
fn a_config_entry_may_name_a_program_to_reload() {
    let catalog =
        ToggleCatalog::parse(&with_check(INI, "reload = \"kwin\"\n"), "test.toml").unwrap();

    assert_eq!(
        catalog.entry("tweak").unwrap().reload(),
        Some(ConfigReload::Kwin)
    );
}

#[test]
fn reload_is_refused_for_unknown_programs_and_other_checks() {
    let unknown = with_check(
        "{ user-json = { path = \".config/a.json\", key = \"K\" } }",
        "reload = \"plasmashell\"\n",
    );

    let package = with_check("{ package = \"a\" }", "reload = \"kwin\"\n");

    assert_eq!(
        problem_of(&unknown).2,
        Problem::UnknownReload("plasmashell".to_owned())
    );
    assert_eq!(problem_of(&package).2, Problem::ReloadWithoutUserConfig);
}

#[test]
fn restart_units_names_no_unit_twice() {
    let text = with_check(
        "{ user-file = { path = \".config/a.conf\", content = \"x = 1\\n\" } }",
        "restart-user-units = [\"pipewire.service\", \"pipewire.service\"]\n",
    );

    assert_eq!(
        problem_of(&text),
        (
            "test.toml".to_owned(),
            "tweak".to_owned(),
            Problem::RestartRepeats("pipewire.service".to_owned())
        )
    );
}
