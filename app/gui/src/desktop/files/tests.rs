use std::ffi::OsString;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_gui_kit::desktop::family::Family;

use super::{DesktopFiles, is_look_file};

/// `~/.config/kdeglobals` of a Garuda Mokka install.
const MOKKA_KDEGLOBALS: &str = toolbox_gui_kit::testing::desktop::kde::INSTALLED_KDEGLOBALS;
/// Garuda's `/etc/skel/.config/gtk-3.0/settings.ini`, which names
/// Sweet-Dark.
const SKEL_GTK3_SETTINGS: &str =
    toolbox_gui_kit::testing::desktop::SKEL_ETC_SKEL_CONFIG_GTK_3_0_SETTINGS_INI;
const SKEL_GTK4_SETTINGS: &str =
    toolbox_gui_kit::testing::desktop::SKEL_ETC_SKEL_CONFIG_GTK_4_0_SETTINGS_INI;
const SKEL_GTK4_CSS: &str = toolbox_gui_kit::testing::desktop::SKEL_ETC_SKEL_CONFIG_GTK_4_0_GTK_CSS;
/// Sweet-Dark's `gtk-4.0/gtk.css`.
const SWEET_DARK: &str = toolbox_gui_kit::testing::desktop::gtk::THEME_GTK_4_0_CSS;

fn files_in(home: &Path) -> DesktopFiles {
    let home = home.to_owned();
    DesktopFiles::from_env(move |key| match key {
        "HOME" => Some(OsString::from(&home)),
        "XDG_DATA_DIRS" => Some(home.join("system").into_os_string()),
        _ => None,
    })
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn kde_reads_kdeglobals_from_the_config_folder() {
    let home = TempDir::new().unwrap();
    write(&home.path().join(".config/kdeglobals"), MOKKA_KDEGLOBALS);

    let desktop = files_in(home.path()).read(Family::Kde).desktop.unwrap();

    assert_eq!(desktop.colors.window.map(|window| window.r()), Some(30));
    assert_eq!(
        desktop.font.map(|font| font.family).as_deref(),
        Some("Inter")
    );
}

#[test]
fn gtk_reads_the_theme_css_then_the_users_css_over_it() {
    let home = TempDir::new().unwrap();
    write(
        &home.path().join(".config/gtk-3.0/settings.ini"),
        SKEL_GTK3_SETTINGS,
    );
    write(
        &home.path().join("system/themes/Sweet-Dark/gtk-4.0/gtk.css"),
        SWEET_DARK,
    );

    // Garuda's own `gtk.css` only imports a file of `_breeze` names, so
    // one definition is added to it.
    write(
        &home.path().join(".config/gtk-4.0/gtk.css"),
        &format!("{SKEL_GTK4_CSS}@define-color accent_bg_color #333333;\n"),
    );

    let desktop = files_in(home.path()).read(Family::Gtk).desktop.unwrap();

    assert_eq!(desktop.colors.window.map(|window| window.r()), Some(0x16));
    assert_eq!(desktop.colors.accent.map(|accent| accent.r()), Some(0x33));
    assert_eq!(desktop.icon_theme.as_deref(), Some("BeautyLine"));
}

#[test]
fn gtk_prefers_the_dark_css_of_a_theme_when_asked_for_dark() {
    let home = TempDir::new().unwrap();
    let theme = home.path().join(".local/share/themes/Sweet-Dark/gtk-4.0");

    write(
        &home.path().join(".config/gtk-4.0/settings.ini"),
        SKEL_GTK4_SETTINGS,
    );
    write(&theme.join("gtk-dark.css"), SWEET_DARK);

    // Sweet-Dark has no light variant, so its CSS is edited into one.
    write(
        &theme.join("gtk.css"),
        &SWEET_DARK.replace(
            "@define-color theme_bg_color #161925;",
            "@define-color theme_bg_color #eeeeee;",
        ),
    );

    let read = files_in(home.path()).read(Family::Gtk);

    assert_eq!(read.prefer_dark, Some(true));
    assert_eq!(
        read.desktop
            .and_then(|desktop| desktop.colors.window)
            .map(|window| window.r()),
        Some(0x16)
    );
}

#[test]
fn missing_files_give_nothing_and_no_error() {
    let home = TempDir::new().unwrap();
    let files = files_in(home.path());

    assert_eq!(files.read(Family::Kde).desktop, None);
    assert_eq!(files.read(Family::Gtk).desktop, None);
    assert_eq!(files.read(Family::Other).desktop, None);
}

#[test]
fn only_the_look_files_count_as_changes() {
    let home = TempDir::new().unwrap();

    assert!(is_look_file(&home.path().join(".config/kdeglobals")));
    assert!(is_look_file(&home.path().join(".config/gtk-4.0/gtk.css")));
    assert!(!is_look_file(&home.path().join(".config/kwinrc")));
}
