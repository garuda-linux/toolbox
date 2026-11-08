use std::ffi::OsString;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_gui_kit::desktop::family::Family;

use super::{DesktopFiles, is_look_file};

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
    write(
        &home.path().join(".config/kdeglobals"),
        "[Icons]\nTheme=breeze-dark\n",
    );

    let read = files_in(home.path()).read(Family::Kde);

    assert_eq!(
        read.desktop
            .and_then(|desktop| desktop.icon_theme)
            .as_deref(),
        Some("breeze-dark")
    );
}

#[test]
fn gtk_reads_the_theme_css_then_the_users_css_over_it() {
    let home = TempDir::new().unwrap();
    write(
        &home.path().join(".config/gtk-3.0/settings.ini"),
        "[Settings]\ngtk-theme-name=Mocha\ngtk-icon-theme-name=Papirus\n",
    );
    write(
        &home.path().join("system/themes/Mocha/gtk-4.0/gtk.css"),
        "@define-color window_bg_color #111111;\n@define-color accent_bg_color #222222;\n",
    );
    write(
        &home.path().join(".config/gtk-4.0/gtk.css"),
        "@define-color accent_bg_color #333333;\n",
    );

    let desktop = files_in(home.path()).read(Family::Gtk).desktop.unwrap();

    assert!(desktop.colors.window.is_some());
    assert_eq!(desktop.colors.accent.map(|accent| accent.r()), Some(0x33));
    assert_eq!(desktop.icon_theme.as_deref(), Some("Papirus"));
}

#[test]
fn gtk_prefers_the_dark_css_of_a_theme_when_asked_for_dark() {
    let home = TempDir::new().unwrap();
    write(
        &home.path().join(".config/gtk-4.0/settings.ini"),
        "[Settings]\ngtk-theme-name=Two\ngtk-application-prefer-dark-theme=true\n",
    );
    write(
        &home.path().join(".local/share/themes/Two/gtk-4.0/gtk.css"),
        "@define-color window_bg_color #eeeeee;\n",
    );
    write(
        &home
            .path()
            .join(".local/share/themes/Two/gtk-4.0/gtk-dark.css"),
        "@define-color window_bg_color #101010;\n",
    );

    let read = files_in(home.path()).read(Family::Gtk);

    assert_eq!(read.prefer_dark, Some(true));
    assert_eq!(
        read.desktop
            .and_then(|desktop| desktop.colors.window)
            .map(|window| window.r()),
        Some(0x10)
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
