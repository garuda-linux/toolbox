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
        .insert(("Papirus".into(), "go-home-symbolic".into()), home);
    lookup
        .files
        .insert(("Papirus".into(), "globe-symbolic".into()), globe);

    let icons = theme_icons("Papirus", &lookup);

    assert_eq!(
        drawn_uri(GO_HOME, &icons),
        "bytes://icons/Papirus/go-home.svg"
    );
    assert_eq!(drawn_uri(GLOBE, &icons), "bytes://icons/bundled/globe.svg");
    assert_eq!(
        drawn_uri(PLUGINS, &icons),
        "bytes://icons/bundled/plugins.svg"
    );
}

#[test]
fn an_unreadable_file_is_left_to_the_bundled_set() {
    let mut lookup = Fake::default();
    lookup.files.insert(
        ("Papirus".into(), "go-home-symbolic".into()),
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
    let home = folder.path().join("go-home-symbolic.svg");
    let globe = folder.path().join("globe-symbolic.svg");
    let padding = "x".repeat(super::MAX_SVG_BYTES);
    fs::write(&home, format!("<svg><!-- {padding} --></svg>")).unwrap();
    fs::write(&globe, "x".repeat(super::MAX_SVG_BYTES)).unwrap();

    let mut lookup = Fake::default();
    lookup
        .files
        .insert(("Papirus".into(), "go-home-symbolic".into()), home);
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
        "bytes://icons/Papirus/globe.svg",
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
fn a_glyph_is_read_from_the_symbolic_name_only() {
    let folder = TempDir::new().unwrap();
    let coloured = folder.path().join("go-home.svg");
    let symbolic = folder.path().join("go-home-symbolic.svg");
    let plugins = folder.path().join("plugins.svg");
    fs::write(&coloured, "<svg/>").unwrap();
    fs::write(&symbolic, r#"<svg fill="currentColor"/>"#).unwrap();
    fs::write(&plugins, "<svg/>").unwrap();

    let mut lookup = Fake::default();
    lookup
        .files
        .insert(("Papirus".into(), "go-home".into()), coloured);
    lookup
        .files
        .insert(("Papirus".into(), "go-home-symbolic".into()), symbolic);
    lookup
        .files
        .insert(("Papirus".into(), "plugins".into()), plugins);

    let icons = theme_icons("Papirus", &lookup);

    assert_eq!(
        drawn_uri(GO_HOME, &icons),
        "bytes://icons/Papirus/go-home.svg"
    );
    assert_eq!(
        GO_HOME
            .image(Some(&icons), Color32::RED)
            .image_options()
            .tint,
        Color32::RED,
        "the symbolic file, which takes the tint, is drawn"
    );
    assert_eq!(
        drawn_uri(PLUGINS, &icons),
        "bytes://icons/bundled/plugins.svg"
    );
}
