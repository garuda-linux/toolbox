use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use egui::{Color32, ImageSource};
use tempfile::TempDir;
use toolbox_gui_kit::icons::{GLOBE, GO_HOME, PLUGINS};

use super::{IconLookup, theme_icons};

#[derive(Default)]
struct Fake {
    files: BTreeMap<(String, String), PathBuf>,
}

impl IconLookup for Fake {
    fn find(&self, theme: &str, name: &str) -> Option<PathBuf> {
        self.files
            .get(&(theme.to_owned(), name.to_owned()))
            .cloned()
    }
}

fn drawn_uri(
    icon: toolbox_gui_kit::icons::Icon,
    icons: &toolbox_gui_kit::icons::ThemeIcons,
) -> String {
    match icon
        .image(Some(icons), Color32::WHITE)
        .source(&egui::Context::default())
    {
        ImageSource::Bytes { uri, bytes: _ } | ImageSource::Uri(uri) => uri.into_owned(),
        ImageSource::Texture(_) => String::new(),
    }
}

#[test]
fn svg_files_of_the_theme_are_read_and_others_are_left_to_the_bundled_set() {
    let folder = TempDir::new().unwrap();
    let home = folder.path().join("go-home.svg");
    let globe = folder.path().join("globe.png");
    fs::write(&home, "<svg/>").unwrap();
    fs::write(&globe, "not svg").unwrap();

    let mut lookup = Fake::default();
    lookup
        .files
        .insert(("Papirus".into(), "go-home".into()), home);
    lookup
        .files
        .insert(("Papirus".into(), "globe-symbolic".into()), globe);

    let icons = theme_icons("Papirus", &lookup);

    assert_eq!(
        drawn_uri(GO_HOME, &icons),
        "bytes://icons/theme/Papirus/go-home.svg"
    );
    assert_eq!(
        drawn_uri(GLOBE, &icons),
        "bytes://icons/bundled/globe-symbolic.svg"
    );
    assert_eq!(
        drawn_uri(PLUGINS, &icons),
        "bytes://icons/bundled/plugins-symbolic.svg"
    );
}

#[test]
fn an_unreadable_file_is_left_to_the_bundled_set() {
    let mut lookup = Fake::default();
    lookup.files.insert(
        ("Papirus".into(), "go-home".into()),
        PathBuf::from("/nonexistent/go-home.svg"),
    );

    let icons = theme_icons("Papirus", &lookup);

    assert_eq!(
        drawn_uri(GO_HOME, &icons),
        "bytes://icons/bundled/go-home.svg"
    );
}

#[test]
fn an_svg_over_the_cap_is_left_to_the_bundled_set() {
    let folder = TempDir::new().unwrap();
    let home = folder.path().join("go-home.svg");
    let globe = folder.path().join("globe-symbolic.svg");
    let padding = "x".repeat(super::MAX_SVG_BYTES);
    fs::write(&home, format!("<svg><!-- {padding} --></svg>")).unwrap();
    fs::write(&globe, "x".repeat(super::MAX_SVG_BYTES)).unwrap();

    let mut lookup = Fake::default();
    lookup
        .files
        .insert(("Papirus".into(), "go-home".into()), home);
    lookup
        .files
        .insert(("Papirus".into(), "globe-symbolic".into()), globe);

    let icons = theme_icons("Papirus", &lookup);

    assert_eq!(
        drawn_uri(GO_HOME, &icons),
        "bytes://icons/bundled/go-home.svg"
    );
    assert_eq!(
        drawn_uri(GLOBE, &icons),
        "bytes://icons/theme/Papirus/globe-symbolic.svg",
        "a file of exactly the cap is still read"
    );
}

#[test]
fn app_pictures_are_found_by_name_in_the_theme() {
    let mut lookup = Fake::default();
    lookup.files.insert(
        ("Tela-circle-dark".into(), "steam".into()),
        PathBuf::from("/icons/Tela/scalable/apps/steam.svg"),
    );

    let names = ["steam".to_owned(), "lutris".to_owned()];
    let icons = super::app_icons("Tela-circle-dark", &names, &lookup);

    assert_eq!(icons.theme(), "Tela-circle-dark");
    assert_eq!(
        icons.get("steam"),
        Some(std::path::Path::new("/icons/Tela/scalable/apps/steam.svg"))
    );
    assert_eq!(icons.get("lutris"), None, "left to the vendored picture");
}

#[test]
fn an_action_icon_is_read_by_its_plain_name_and_a_glyph_by_its_symbolic_one() {
    let folder = TempDir::new().unwrap();
    let action = folder.path().join("go-home.svg");
    let symbolic = folder.path().join("plugins-symbolic.svg");
    let plain = folder.path().join("plugins.svg");
    fs::write(&action, r#"<svg fill="currentColor"/>"#).unwrap();
    fs::write(&symbolic, r#"<svg fill="currentColor"/>"#).unwrap();
    fs::write(&plain, "<svg/>").unwrap();

    let mut lookup = Fake::default();
    lookup
        .files
        .insert(("Papirus".into(), "go-home".into()), action);
    lookup
        .files
        .insert(("Papirus".into(), "plugins-symbolic".into()), symbolic);
    lookup
        .files
        .insert(("Papirus".into(), "plugins".into()), plain);

    let icons = theme_icons("Papirus", &lookup);

    assert_eq!(
        drawn_uri(GO_HOME, &icons),
        "bytes://icons/theme/Papirus/go-home.svg"
    );
    assert_eq!(
        GO_HOME
            .image(Some(&icons), Color32::RED)
            .image_options()
            .tint,
        Color32::RED,
        "the action icon takes the tint, as Breeze draws it in the text colour"
    );
    assert_eq!(
        drawn_uri(PLUGINS, &icons),
        "bytes://icons/theme/Papirus/plugins-symbolic.svg",
        "a glyph is read by its symbolic name, not the plain file"
    );
}
