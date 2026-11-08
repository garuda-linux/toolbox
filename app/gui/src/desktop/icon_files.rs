//! Reads the desktop icon theme's files off the UI thread: the theme's
//! glyphs for the app's icons, and the theme's pictures for the apps the
//! catalogs offer.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use toolbox_gui_kit::icons::{ALL, ThemeIcons};

/// Where an icon of a theme is on disk. A port, so tests need no icon
/// theme installed: `freedesktop-icons` reads its folders from the process
/// environment.
pub trait IconLookup {
    fn find(&self, theme: &str, name: &str) -> Option<PathBuf>;
}

/// The installed themes, after the icon theme spec.
#[derive(Clone, Copy, Debug)]
pub struct SystemIcons {
    size: u16,
    svg_only: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppIcons {
    theme: String,
    files: BTreeMap<String, PathBuf>,
}

impl SystemIcons {
    /// Breeze's toolbar and sidebar size, which themes draw most carefully,
    /// and SVG only, so a glyph stays sharp at every scale.
    pub const GLYPHS: Self = Self {
        size: 22,
        svg_only: true,
    };

    /// App pictures at the size the rows show, in any format the theme has.
    pub const APPS: Self = Self {
        size: crate::pages::rows::PICTURE_PIXELS,
        svg_only: false,
    };
}

impl IconLookup for SystemIcons {
    fn find(&self, theme: &str, name: &str) -> Option<PathBuf> {
        let lookup = freedesktop_icons::lookup(name)
            .with_theme(theme)
            .with_size(self.size);

        if self.svg_only {
            lookup.force_svg().find()
        } else {
            lookup.find()
        }
    }
}

impl AppIcons {
    pub fn theme(&self) -> &str {
        &self.theme
    }

    pub fn get(&self, name: &str) -> Option<&Path> {
        self.files.get(name).map(PathBuf::as_path)
    }
}

/// The SVG files `theme` has for the app's icons. A PNG of the theme is
/// left to the bundled glyph, which stays sharp.
pub fn theme_icons(theme: &str, lookup: &impl IconLookup) -> ThemeIcons {
    let mut icons = ThemeIcons::new(theme);
    for icon in ALL {
        let file = icon
            .theme_name()
            .and_then(|name| lookup.find(theme, &name))
            .filter(|path| path.extension().is_some_and(|ending| ending == "svg"))
            .and_then(|path| fs::read(path).ok());

        if let Some(file) = file {
            icons.insert(icon.name(), Arc::from(file));
        }
    }

    icons
}

/// Where `theme` (or the themes it inherits, hicolor last) has a picture
/// for each of `names`.
pub fn app_icons(theme: &str, names: &[String], lookup: &impl IconLookup) -> AppIcons {
    AppIcons {
        theme: theme.to_owned(),
        files: names
            .iter()
            .filter_map(|name| Some((name.clone(), lookup.find(theme, name)?)))
            .collect(),
    }
}
