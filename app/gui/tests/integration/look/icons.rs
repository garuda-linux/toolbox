//! The user's Tela icon themes for the guide pictures. The files are the ones
//! the desktop's lookup returns for the app's icons, so the app reads them as
//! it reads the desktop's.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::Theme;
use toolbox_gui::desktop::icon_files::{IconLookup, theme_icons};
use toolbox_gui_kit::icons::ThemeIcons;

/// The vendored themes are read as the desktop reads them: the theme's own
/// files first, then the themes its `Inherits` line names, in order.
struct Installed;

impl IconLookup for Installed {
    fn find(&self, theme: &str, name: &str) -> Option<PathBuf> {
        let file = format!("{name}.svg");
        std::iter::once(theme.to_owned())
            .chain(inherits(theme))
            .find_map(|folder| find_file(&root().join(folder), OsStr::new(&file)))
    }
}

/// One folder per theme, laid out as the installed theme is.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/integration/look/fixtures/installed/icons")
}

/// The icons of the user's desktop theme for `theme`: the dark pictures show
/// Tela circle dark, the light ones Tela circle light.
pub fn for_theme(theme: Theme) -> Arc<ThemeIcons> {
    let name = match theme {
        Theme::Dark => "Tela-circle-dark",
        Theme::Light => "Tela-circle-light",
    };

    Arc::new(theme_icons(name, &Installed))
}

/// The themes named by the `Inherits` line of `theme`'s `index.theme`.
fn inherits(theme: &str) -> Vec<String> {
    let Ok(index) = fs::read_to_string(root().join(theme).join("index.theme")) else {
        return Vec::new();
    };

    index
        .lines()
        .find_map(|line| line.strip_prefix("Inherits="))
        .map(|names| names.split(',').map(str::to_owned).collect())
        .unwrap_or_default()
}

/// The first file called `file` below `dir`, searched depth first.
fn find_file(dir: &Path, file: &OsStr) -> Option<PathBuf> {
    let mut folders = Vec::new();
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            folders.push(path);
            continue;
        }

        if path.file_name() == Some(file) {
            return Some(path);
        }
    }

    folders.iter().find_map(|folder| find_file(folder, file))
}

/// The URI `icon` draws from in `theme`'s vendored Tela copy.
fn drawn_from(icon: toolbox_gui_kit::icons::Icon, theme: Theme) -> String {
    let icons = for_theme(theme);
    match icon
        .image(Some(&icons), egui::Color32::WHITE)
        .source(&egui::Context::default())
    {
        egui::ImageSource::Bytes { uri, bytes: _ } | egui::ImageSource::Uri(uri) => {
            uri.into_owned()
        }
        egui::ImageSource::Texture(_) => String::new(),
    }
}

/// The empty states of Network, Languages and others draw `help-about`,
/// which Tela circle has only as a colour app icon.
#[test]
fn tela_circle_leaves_help_about_to_the_bundled_glyph_and_keeps_its_own_actions() {
    use toolbox_gui_kit::icons::{GO_HOME, HELP_ABOUT};

    for theme in [Theme::Dark, Theme::Light] {
        assert_eq!(
            drawn_from(HELP_ABOUT, theme),
            "bytes://icons/bundled/help-about.svg"
        );
        assert!(drawn_from(GO_HOME, theme).starts_with("bytes://icons/theme/Tela-circle-"));
    }
}
