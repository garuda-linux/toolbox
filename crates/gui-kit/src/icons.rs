//! Icons by freedesktop name, in two kinds.
//!
//! A glyph is a monochrome symbol the text colour tints, for buttons,
//! fields, menus and row marks. A colour icon is Breeze's own colour file,
//! drawn as it is (`colour`), for pages, status and rows. Both ask the
//! desktop's icon theme for their file by its theme name (a glyph's
//! `-symbolic` one), and each stands on a bundled file when the theme has
//! none. Each constant embeds its bundled file, so a missing one fails the
//! build, as Rerun's `re_ui` icons do.

mod all;
pub mod colour;
mod ink;
mod loader;
mod mask;
mod picture;
pub mod svg;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;

use egui::{Color32, Context, Id, Image};

pub use self::all::ALL;
pub use self::colour::{
    DIALOG_ERROR, DIALOG_INFORMATION, DIALOG_POSITIVE, DIALOG_WARNING,
    PREFERENCES_VIRTUALIZATION_CONTAINER, SYSTEM_SOFTWARE_INSTALL,
};
use self::ink::inked_image;
use self::loader::THEME_URI;
pub use self::loader::{ThemeLoader, warm};
use self::mask::mask;
pub use self::picture::picture_uri;

macro_rules! bundled {
    ($name:literal) => {
        Icon {
            name: $name,
            theme: concat!($name, "-symbolic"),
            bundled: include_bytes!(concat!("../assets/icons/", $name, ".svg")),
            tinted: true,
        }
    };
}

/// A monochrome action icon Breeze draws in `currentColor`. KDE shows its
/// own in the text colour, so the theme's plain file is tinted too.
macro_rules! action {
    ($name:literal) => {
        Icon {
            name: $name,
            theme: $name,
            bundled: include_bytes!(concat!("../assets/icons/", $name, ".svg")),
            tinted: true,
        }
    };
}

pub const APPLICATIONS_DEVELOPMENT: Icon = bundled!("applications-development");
pub const APPLICATIONS_GAMES: Icon = bundled!("applications-games");
pub const APPLICATIONS_GRAPHICS: Icon = bundled!("applications-graphics");
pub const APPLICATIONS_INTERNET: Icon = bundled!("applications-internet");
pub const APPLICATIONS_MULTIMEDIA: Icon = bundled!("applications-multimedia");
pub const APPLICATIONS_OFFICE: Icon = bundled!("applications-office");
pub const APPLICATIONS_OTHER: Icon = bundled!("applications-other");
pub const APPLICATIONS_SYSTEM: Icon = bundled!("applications-system");
pub const AUDIO_VOLUME_HIGH: Icon = bundled!("audio-volume-high");
pub const BATTERY_GOOD: Icon = bundled!("battery-good");
pub const CAMERA_VIDEO: Icon = bundled!("camera-video");
pub const CHECKBOX: Icon = bundled!("checkbox");
pub const CHECKMARK: Icon = bundled!("checkmark");
pub const CHRONOMETER: Icon = bundled!("chronometer");
pub const COMPUTER: Icon = bundled!("computer");
pub const CONFIGURE: Icon = action!("configure");
pub const DESKTOP: Icon = bundled!("desktop");
pub const DIALOG_CANCEL: Icon = bundled!("dialog-cancel");
pub const DIALOG_MESSAGES: Icon = bundled!("dialog-messages");
pub const DIALOG_OK: Icon = bundled!("dialog-ok");
pub const DIALOG_OK_APPLY: Icon = bundled!("dialog-ok-apply");
pub const DIALOG_SCRIPTS: Icon = bundled!("dialog-scripts");
pub const DOCUMENTATION: Icon = bundled!("documentation");
pub const DOCUMENT_PROPERTIES: Icon = bundled!("document-properties");
pub const DOCUMENT_PREVIEW: Icon = bundled!("document-preview");
pub const DOCUMENT_REVERT: Icon = bundled!("document-revert");
pub const DRAW_CROSS: Icon = bundled!("draw-cross");
pub const DRIVE_HARDDISK: Icon = bundled!("drive-harddisk");
pub const DRIVE_HARDDISK_ROOT: Icon = bundled!("drive-harddisk-root");
pub const EDIT_CLEAR_ALL: Icon = bundled!("edit-clear-all");
pub const EDIT_COPY: Icon = bundled!("edit-copy");
pub const EDIT_DELETE: Icon = bundled!("edit-delete");
pub const EDIT_FIND: Icon = action!("edit-find");
pub const ERROR: Icon = bundled!("error");
pub const FOLDER_DOWNLOAD: Icon = bundled!("folder-download");
pub const FORMAT_LIST_UNORDERED: Icon = bundled!("format-list-unordered");
pub const GAMES_CONFIG_BOARD: Icon = bundled!("games-config-board");
pub const GAMES_HIGHSCORES: Icon = bundled!("games-highscores");
pub const GAMES_HINT: Icon = bundled!("games-hint");
pub const GAMES_SOLVE: Icon = bundled!("games-solve");
pub const GLOBE: Icon = bundled!("globe");
pub const GO_NEXT: Icon = bundled!("go-next");
pub const GO_PREVIOUS: Icon = action!("go-previous");
pub const GO_DOWN: Icon = bundled!("go-down");
pub const GO_HOME: Icon = action!("go-home");
pub const HELP_ABOUT: Icon = action!("help-about");
pub const HELP_CONTENTS: Icon = action!("help-contents");
pub const INPUT_GAMEPAD: Icon = bundled!("input-gamepad");
pub const INPUT_KEYBOARD: Icon = bundled!("input-keyboard");
pub const INPUT_TOUCHPAD: Icon = bundled!("input-touchpad");
pub const INTERNET_MAIL: Icon = bundled!("internet-mail");
pub const INTERNET_WEB_BROWSER: Icon = bundled!("internet-web-browser");
pub const LANGUAGES: Icon = bundled!("languages");
pub const MEDIA_PLAYBACK_START: Icon = bundled!("media-playback-start");
pub const MEDIA_PLAYBACK_STOP: Icon = bundled!("media-playback-stop");
pub const MEDIA_PLAYLIST_REPEAT: Icon = bundled!("media-playlist-repeat");
pub const NETWORK_CONNECT: Icon = bundled!("network-connect");
pub const NETWORK_SERVER: Icon = bundled!("network-server");
pub const NETWORK_WIRED: Icon = bundled!("network-wired");
pub const NETWORK_WIRELESS: Icon = bundled!("network-wireless");
pub const NETWORK_WIRELESS_SIGNAL_GOOD: Icon = bundled!("network-wireless-signal-good");
pub const OBJECT_LOCKED: Icon = bundled!("object-locked");
pub const OBJECT_UNLOCKED: Icon = bundled!("object-unlocked");
pub const OFFICE_REPORT: Icon = bundled!("office-report");
pub const OVERFLOW_MENU: Icon = bundled!("overflow-menu");
pub const PACKAGE: Icon = bundled!("package");
pub const PLUGINS: Icon = bundled!("plugins");
pub const PREFERENCES_DESKTOP_THEME_GLOBAL: Icon = bundled!("preferences-desktop-theme-global");
pub const PREFERENCES_SYSTEM_BLUETOOTH: Icon = bundled!("preferences-system-bluetooth");
pub const PRINTER: Icon = bundled!("printer");
pub const REPOSITORY: Icon = bundled!("repository");
pub const RUN_BUILD: Icon = bundled!("run-build");
pub const RUN_BUILD_CONFIGURE: Icon = bundled!("run-build-configure");
pub const RUN_INSTALL: Icon = bundled!("run-install");
pub const SECURITY_HIGH: Icon = bundled!("security-high");
pub const SIDEBAR_COLLAPSE_LEFT: Icon = bundled!("sidebar-collapse-left");
pub const SIDEBAR_EXPAND_LEFT: Icon = bundled!("sidebar-expand-left");
pub const SPEEDOMETER: Icon = bundled!("speedometer");
pub const SYSTEM_REBOOT: Icon = bundled!("system-reboot");
pub const SYSTEM_RUN: Icon = bundled!("system-run");
pub const SYSTEM_SEARCH: Icon = bundled!("system-search");
pub const SYSTEM_SOFTWARE_UPDATE: Icon = bundled!("system-software-update");
pub const TEXT_X_GENERIC: Icon = bundled!("text-x-generic");
pub const TOOLS: Icon = bundled!("tools");
pub const TOOLS_WIZARD: Icon = bundled!("tools-wizard");
pub const USER_OTHERS: Icon = bundled!("user-others");
pub const UTILITIES_TERMINAL: Icon = bundled!("utilities-terminal");
pub const VCS_DIFF: Icon = bundled!("vcs-diff");
pub const VIEW_CERTIFICATE: Icon = bundled!("view-certificate");
pub const VIEW_HIDDEN: Icon = bundled!("view-hidden");
pub const VIEW_LIST_DETAILS: Icon = bundled!("view-list-details");
pub const VIEW_LIST_ICONS: Icon = bundled!("view-list-icons");
pub const VIEW_MEDIA_EQUALIZER: Icon = bundled!("view-media-equalizer");
pub const VIEW_PIM_TASKS: Icon = bundled!("view-pim-tasks");
pub const VIEW_REFRESH: Icon = bundled!("view-refresh");
pub const VIEW_TASK: Icon = bundled!("view-task");
pub const VIEW_VISIBLE: Icon = bundled!("view-visible");
pub const WINDOW_CLOSE: Icon = bundled!("window-close");
pub const ZOOM_IN: Icon = bundled!("zoom-in");
pub const ZOOM_ORIGINAL: Icon = bundled!("zoom-original");
pub const ZOOM_OUT: Icon = bundled!("zoom-out");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    name: &'static str,
    /// The file's name in the desktop theme, which also keys its bundled
    /// copy, so a glyph and a colour icon of one name never collide.
    theme: &'static str,
    bundled: &'static [u8],
    /// A monochrome glyph: its file is a mask the tint colours. A colour
    /// icon keeps its colours.
    tinted: bool,
}

/// The icon files a desktop theme has for the app's icons, read off the
/// UI thread, keyed by theme name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeIcons {
    theme: String,
    files: BTreeMap<&'static str, ThemeFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ThemeFile {
    svg: Arc<[u8]>,
    tinted: bool,
}

impl Icon {
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn bundled(&self) -> &'static [u8] {
        self.bundled
    }

    /// The file to ask for in the desktop theme.
    pub const fn theme_name(&self) -> &'static str {
        self.theme
    }

    /// Whether the icon is a monochrome glyph that takes the tint.
    pub const fn is_glyph(&self) -> bool {
        self.tinted
    }

    /// The colour icon a row draws for this glyph (`colour::of_row`), or
    /// the glyph itself where Breeze has no colour file for it.
    #[must_use = "the row icon is what gets drawn"]
    pub fn for_row(self) -> Self {
        if self.is_glyph() {
            colour::of_row(self)
        } else {
            self
        }
    }

    /// The theme's file as it is, or the bundled one. A glyph takes
    /// `tint`. A colour icon keeps its colours, and its text parts (the
    /// `ColorScheme-Text` class and `currentColor`) take `tint` as the ink,
    /// so a selected entry shows its own text colour. The caller adds the
    /// alt text, see the `UiExt` icon widgets.
    #[must_use = "an image does nothing until it is added to a ui"]
    pub fn image(&self, theme: Option<&ThemeIcons>, tint: Color32) -> Image<'static> {
        let themed = theme.and_then(|theme| {
            let file = theme.files.get(self.theme)?;
            if file.tinted {
                let uri = format!("{THEME_URI}{}/{}.svg", theme.theme, self.theme);
                return Some(Image::from_bytes(uri, Arc::clone(&file.svg)).tint(tint));
            }

            let stem = format!("{THEME_URI}{}/{}", theme.theme, self.theme);
            Some(inked_image(&stem, &file.svg, tint))
        });

        themed.unwrap_or_else(|| {
            if self.tinted {
                let uri = format!("bytes://icons/bundled/{}.svg", self.theme);
                Image::from_bytes(uri, self.bundled).tint(tint)
            } else {
                let stem = format!("bytes://icons/bundled/{}", self.theme);
                inked_image(&stem, self.bundled, tint)
            }
        })
    }
}

impl ThemeIcons {
    pub fn new(theme: &str) -> Self {
        Self {
            theme: theme.to_owned(),
            files: BTreeMap::new(),
        }
    }

    /// A glyph's file is a mask: white where it paints, so the tint leaves
    /// only shape and alpha (a symbolic shield would keep a green check). A
    /// colour icon keeps its colours, and a `currentColor` part of it takes
    /// the ink, as in the bundled copies.
    pub fn insert(&mut self, icon: Icon, file: Arc<[u8]>) {
        let text = String::from_utf8_lossy(&file);
        let svg = if icon.tinted {
            Arc::from(mask(&text).into_bytes())
        } else {
            file
        };

        self.files.insert(
            icon.theme,
            ThemeFile {
                svg,
                tinted: icon.tinted,
            },
        );
    }

    pub fn theme(&self) -> &str {
        &self.theme
    }
}

/// The app's icon of that freedesktop name, for names that come as data
/// (the catalogs' section icons). A glyph wins over a colour icon of the
/// same name.
pub fn named(name: &str) -> Option<Icon> {
    let same = || ALL.into_iter().filter(|icon| icon.name == name);
    same().find(Icon::is_glyph).or_else(|| same().next())
}

/// Makes `icons` the desktop theme every icon widget draws from. `None`
/// leaves the bundled glyphs.
pub fn use_theme(ctx: &Context, icons: Option<Arc<ThemeIcons>>) {
    ctx.data_mut(|data| match icons {
        Some(icons) => {
            data.insert_temp(theme_id(), icons);
        }
        None => data.remove::<Arc<ThemeIcons>>(theme_id()),
    });
}

pub fn theme_in_use(ctx: &Context) -> Option<Arc<ThemeIcons>> {
    ctx.data(|data| data.get_temp(theme_id()))
}

fn theme_id() -> Id {
    Id::unique("toolbox_gui_kit::icons::theme")
}
