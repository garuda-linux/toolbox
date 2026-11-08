use std::ffi::OsString;

use clap_complete::engine::complete;

use super::{Names, PLACES, command, matching, registration, wanted};

fn fake(names: Names) -> Vec<String> {
    let found: &[&str] = match names {
        Names::Switches => &["pipewire-support", "bluetooth-support", "pipewire-support"],
        Names::Services { user: false } => &["sshd.service", "bluetooth.service"],
        Names::Services { user: true } => &["pipewire.service"],
        Names::Pages => &["maintenance", "kernels"],
        Names::Shells => &["/bin/sh", "/usr/bin/fish"],
        Names::AppCategories => &["internet", "games"],
        Names::BootKeys => &["default", "quiet", "timeout"],
        Names::SettingKeys => &["language", "log-level"],
        Names::OnOff => &["on", "off"],
        Names::LogLevels => &["debug", "info"],
        Names::Apps => panic!("a broken source"),
        Names::OfferedPackages => &["gimp", "git", "firefox"],
        Names::InstalledPackages => &["git", "glibc"],
        Names::AurPackages => &["paru", "paru-bin"],
        Names::LanguagePacks { installed: false } => &["firefox-i18n-de"],
        Names::LanguagePacks { installed: true } => &["thunderbird-i18n-de"],
        Names::GuidePages => &["kernels", "packages"],
        Names::SetupChoices => &["firefox", "strawberry-player"],
        Names::KernelParameters => &["quiet", "nowatchdog"],
        Names::Kernels
        | Names::Locales
        | Names::Users
        | Names::Groups
        | Names::ConfigFiles
        | Names::Tasks
        | Names::BootEntries
        | Names::Drivers
        | Names::BootValues
        | Names::SettingValues
        | Names::ColourSchemes { dark: _ } => &[],
    };

    found.iter().map(|name| (*name).to_owned()).collect()
}

fn completed(line: &[&str]) -> Vec<String> {
    let mut cmd = command(fake);
    cmd.build();
    let args: Vec<OsString> = line.iter().map(OsString::from).collect();

    complete(&mut cmd, args, line.len() - 1, None)
        .unwrap()
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn every_place_names_a_real_argument() {
    let mut top = crate::command();
    top.build();

    for (path, arg, _) in PLACES {
        let sub = path.iter().fold(&top, |command, name| {
            command
                .find_subcommand(name)
                .unwrap_or_else(|| panic!("no subcommand {path:?}"))
        });

        assert!(
            sub.get_arguments().any(|known| known.get_id() == arg),
            "{path:?} has no argument {arg}"
        );
    }
}

#[test]
fn an_argument_completes_the_names_of_its_kind() {
    assert_eq!(
        completed(&["garuda-toolbox", "toggle", "on", "pipe"]),
        ["pipewire-support"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "--page", "ma"]),
        ["maintenance"]
    );
}

#[test]
fn a_broken_source_completes_nothing() {
    assert_eq!(
        completed(&["garuda-toolbox", "apps", "install", "fi"]),
        Vec::<String>::new()
    );
}

#[test]
fn user_services_come_with_user() {
    let words = ["services", "start", "--user", "pi"].map(String::from);

    assert_eq!(
        wanted(Names::Services { user: false }, &words),
        Some(Names::Services { user: true })
    );
    assert_eq!(
        wanted(Names::Services { user: false }, &words[..2]),
        Some(Names::Services { user: false })
    );
}

#[test]
fn boot_entries_only_for_the_default_entry() {
    let default = ["boot", "set", "default", ""].map(String::from);
    let timeout = ["boot", "set", "timeout", ""].map(String::from);

    assert_eq!(
        wanted(Names::BootEntries, &default),
        Some(Names::BootEntries)
    );
    assert_eq!(wanted(Names::BootEntries, &timeout), None);
}

#[test]
fn categories_shells_and_keys_complete() {
    assert_eq!(
        completed(&["garuda-toolbox", "apps", "list", "--category", "in"]),
        ["internet"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "users", "add", "ada", "--shell", "/usr"]),
        ["/usr/bin/fish"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "boot", "set", "qu"]),
        ["quiet"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "settings", "set", "la"]),
        ["language"]
    );
}

#[test]
fn a_boot_setting_offers_its_own_values() {
    let quiet = ["boot", "set", "quiet", ""].map(String::from);
    let timeout = ["boot", "set", "timeout", ""].map(String::from);

    assert_eq!(wanted(Names::BootValues, &quiet), Some(Names::OnOff));
    assert_eq!(wanted(Names::BootValues, &timeout), None);
}

#[test]
fn a_setting_offers_its_own_values() {
    let log = ["settings", "set", "log-level", ""].map(String::from);
    let language = ["settings", "set", "language", ""].map(String::from);

    assert_eq!(wanted(Names::SettingValues, &log), Some(Names::LogLevels));
    assert_eq!(wanted(Names::SettingValues, &language), None);

    let dark = ["settings", "set", "dark-scheme", ""].map(String::from);
    let light = ["settings", "set", "light-scheme", ""].map(String::from);
    assert_eq!(
        wanted(Names::SettingValues, &dark),
        Some(Names::ColourSchemes { dark: true })
    );
    assert_eq!(
        wanted(Names::SettingValues, &light),
        Some(Names::ColourSchemes { dark: false })
    );
}

#[test]
fn matching_names_start_with_the_word_and_come_once() {
    let found = ["linux", "linux-lts", "linux", "zfs"]
        .map(String::from)
        .to_vec();

    assert_eq!(matching(found, "lin"), ["linux", "linux-lts"]);
}

#[test]
fn install_offers_the_repositories_and_remove_the_installed() {
    assert_eq!(
        completed(&["garuda-toolbox", "packages", "install", "gi"]),
        ["gimp", "git"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "packages", "remove", "g"]),
        ["git", "glibc"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "packages", "show", "gl"]),
        ["glibc"]
    );
}

#[test]
fn the_aur_gets_its_own_names() {
    let aur = ["packages", "install", "--aur", ""].map(String::from);
    let show = ["packages", "show", "--aur", ""].map(String::from);
    let repo = ["packages", "install", ""].map(String::from);

    assert_eq!(
        wanted(Names::OfferedPackages, &aur),
        Some(Names::AurPackages)
    );
    assert_eq!(
        wanted(Names::InstalledPackages, &show),
        Some(Names::AurPackages)
    );
    assert_eq!(
        wanted(Names::OfferedPackages, &repo),
        Some(Names::OfferedPackages)
    );
}

#[test]
fn packs_pages_choices_and_parameters_complete() {
    assert_eq!(
        completed(&["garuda-toolbox", "locales", "packs", "install", "fi"]),
        ["firefox-i18n-de"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "locales", "packs", "remove", "th"]),
        ["thunderbird-i18n-de"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "guide", "show", "ke"]),
        ["kernels"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "setup", "apply", "st"]),
        ["strawberry-player"]
    );
    assert_eq!(
        completed(&["garuda-toolbox", "boot", "param", "remove", "no"]),
        ["nowatchdog"]
    );
}

#[test]
fn the_registration_asks_the_binary_for_the_names() {
    for shell in [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Elvish,
        clap_complete::Shell::Fish,
        clap_complete::Shell::PowerShell,
        clap_complete::Shell::Zsh,
    ] {
        let mut text = Vec::new();
        registration(shell, &mut text);
        let text = String::from_utf8(text).unwrap();

        assert!(text.contains("COMPLETE"), "{shell} sets no COMPLETE");
        assert!(text.contains("garuda-toolbox"), "{shell} calls no binary");
    }
}

#[test]
fn the_zsh_registration_answers_its_first_tab_when_autoloaded() {
    let mut text = Vec::new();
    registration(clap_complete::Shell::Zsh, &mut text);
    let text = String::from_utf8(text).unwrap();

    assert!(text.starts_with("#compdef garuda-toolbox"));
    assert!(text.contains("_clap_dynamic_completer_garuda_toolbox \"$@\""));
}
