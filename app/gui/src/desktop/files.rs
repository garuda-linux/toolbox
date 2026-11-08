//! The desktop's look files on disk: where they are, reading them, and
//! which file names count as a change worth reading again.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use toolbox_gui_kit::desktop::family::Family;
use toolbox_gui_kit::desktop::gtk::settings::{self, Settings};
use toolbox_gui_kit::desktop::{DesktopTheme, gtk, kde};

/// The file names whose change can change the look.
const LOOK_FILES: [&str; 4] = ["kdeglobals", "gtk.css", "gtk-dark.css", "settings.ini"];
/// GTK reads the newest folder first.
const GTK_VERSIONS: [&str; 2] = ["gtk-4.0", "gtk-3.0"];

/// The XDG folders the look files live in.
#[derive(Clone, Debug)]
pub struct DesktopFiles {
    config: PathBuf,
    data_home: PathBuf,
    home: PathBuf,
    data_dirs: Vec<PathBuf>,
}

/// What the files say, before the portal is added.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub desktop: Option<DesktopTheme>,
    pub prefer_dark: Option<bool>,
}

impl DesktopFiles {
    #[must_use = "the paths are only computed, nothing is read"]
    pub fn from_process() -> Self {
        Self::from_env(|key| std::env::var_os(key))
    }

    /// `lookup` answers environment variables, so tests never read the
    /// real ones.
    #[must_use = "the paths are only computed, nothing is read"]
    pub fn from_env(lookup: impl Fn(&str) -> Option<OsString>) -> Self {
        let home = toolbox_config::Paths::from_env(&lookup)
            .home()
            .to_path_buf();

        let absolute = |key: &str| {
            lookup(key)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };

        let data_dirs = lookup("XDG_DATA_DIRS")
            .map(|dirs| {
                std::env::split_paths(&dirs)
                    .filter(|dir| dir.is_absolute())
                    .collect::<Vec<_>>()
            })
            .filter(|dirs| !dirs.is_empty())
            .unwrap_or_else(|| vec!["/usr/local/share".into(), "/usr/share".into()]);

        Self {
            config: absolute("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config")),
            data_home: absolute("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share")),
            home,
            data_dirs,
        }
    }

    /// Reads what `family` keeps its look in. A missing or unreadable file
    /// counts as not there.
    pub fn read(&self, family: Family) -> Read {
        match family {
            Family::Kde => Read {
                desktop: read_text(&self.config.join("kdeglobals")).map(|text| kde::parse(&text)),
                prefer_dark: None,
            },
            Family::Gtk => self.read_gtk(),
            Family::Other => Read::default(),
        }
    }

    /// The folders to watch for `family`. The files are replaced by
    /// rename, so their folders are watched, not the files.
    pub fn watched_dirs(&self, family: Family) -> Vec<PathBuf> {
        match family {
            Family::Kde => vec![self.config.clone()],
            Family::Gtk => GTK_VERSIONS
                .iter()
                .map(|version| self.config.join(version))
                .collect(),
            Family::Other => Vec::new(),
        }
    }

    fn read_gtk(&self) -> Read {
        let settings = GTK_VERSIONS
            .iter()
            .filter_map(|version| read_text(&self.config.join(version).join("settings.ini")))
            .map(|text| settings::parse(&text))
            .reduce(merged)
            .unwrap_or_default();

        let theme_css = settings
            .theme
            .as_deref()
            .and_then(|name| self.theme_css(name, settings.prefer_dark == Some(true)));

        let user_css = read_text(&self.config.join("gtk-4.0/gtk.css"));
        let found = theme_css.is_some()
            || user_css.is_some()
            || settings.font.is_some()
            || settings.icon_theme.is_some()
            || settings.enable_animations.is_some();

        let css = [theme_css, user_css]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");

        Read {
            desktop: found.then(|| DesktopTheme {
                colors: gtk::parse(&css),
                font: settings.font,
                fixed_font: None,
                icon_theme: settings.icon_theme,
                animations: settings.enable_animations,
            }),
            prefer_dark: settings.prefer_dark,
        }
    }

    /// The `gtk.css` of the theme `name`, in GTK's search order: the
    /// user's folders, then the system's.
    fn theme_css(&self, name: &str, dark: bool) -> Option<String> {
        if name.contains('/') || name.starts_with('.') {
            return None;
        }

        let files: &[&str] = if dark {
            &["gtk-dark.css", "gtk.css"]
        } else {
            &["gtk.css"]
        };

        let mut bases = vec![self.data_home.join("themes"), self.home.join(".themes")];
        bases.extend(self.data_dirs.iter().map(|dir| dir.join("themes")));

        bases.iter().find_map(|base| {
            GTK_VERSIONS.iter().find_map(|version| {
                let folder = base.join(name).join(version);
                files.iter().find_map(|file| read_text(&folder.join(file)))
            })
        })
    }
}

pub fn is_look_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| LOOK_FILES.contains(&name))
}

/// `first` with the gaps filled from `second`.
fn merged(first: Settings, second: Settings) -> Settings {
    Settings {
        theme: first.theme.or(second.theme),
        font: first.font.or(second.font),
        icon_theme: first.icon_theme.or(second.icon_theme),
        prefer_dark: first.prefer_dark.or(second.prefer_dark),
        enable_animations: first.enable_animations.or(second.enable_animations),
    }
}

fn read_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}
