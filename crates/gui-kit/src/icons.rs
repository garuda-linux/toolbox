//! Icons by freedesktop name.
//!
//! The desktop's icon theme draws them where it has them. The bundled
//! Breeze glyphs, tinted with the text colour, stand in everywhere else.
//! Each constant embeds its bundled file, so a missing one fails the
//! build, as Rerun's `re_ui` icons do.

mod loader;
mod mask;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;

use egui::{Color32, Context, Id, Image};

use self::loader::THEME_URI;
pub use self::loader::{ThemeLoader, warm};
use self::mask::mask;

macro_rules! bundled {
    ($name:literal) => {
        Icon {
            name: $name,
            bundled: include_bytes!(concat!("../assets/icons/", $name, ".svg")),
            tinted: true,
            themed: true,
        }
    };
}

/// A bundled icon with its own colours, which a tint would muddy. Only
/// for a Breeze icon without a symbolic variant.
macro_rules! coloured {
    ($name:literal) => {
        Icon {
            name: $name,
            bundled: include_bytes!(concat!("../assets/icons/", $name, ".svg")),
            tinted: false,
            themed: true,
        }
    };
}

/// A state glyph (info, warning, error, success). Themes draw these as
/// coloured shapes, so the bundled glyph keeps every banner, toast and empty
/// state alike. It is the bare symbol, as `widgets::status_tile` frames it.
macro_rules! state {
    ($name:literal) => {
        Icon {
            name: $name,
            bundled: include_bytes!(concat!("../assets/icons/", $name, ".svg")),
            tinted: true,
            themed: false,
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
pub const CHECKMARK: Icon = bundled!("checkmark");
pub const CHRONOMETER: Icon = bundled!("chronometer");
pub const COMPUTER: Icon = bundled!("computer");
pub const CONFIGURE: Icon = bundled!("configure");
pub const DESKTOP: Icon = bundled!("desktop");
pub const DIALOG_ERROR: Icon = state!("dialog-error");
pub const DIALOG_POSITIVE: Icon = state!("dialog-positive");
pub const DIALOG_MESSAGES: Icon = bundled!("dialog-messages");
pub const DIALOG_WARNING: Icon = state!("dialog-warning");
pub const DOCUMENTATION: Icon = bundled!("documentation");
pub const DOCUMENT_PROPERTIES: Icon = bundled!("document-properties");
pub const DOCUMENT_PREVIEW: Icon = bundled!("document-preview");
pub const DOCUMENT_REVERT: Icon = bundled!("document-revert");
pub const DRAW_CROSS: Icon = bundled!("draw-cross");
pub const DRIVE_HARDDISK_ROOT: Icon = bundled!("drive-harddisk-root");
pub const EDIT_CLEAR_ALL: Icon = bundled!("edit-clear-all");
pub const EDIT_COPY: Icon = bundled!("edit-copy");
pub const FORMAT_LIST_UNORDERED: Icon = bundled!("format-list-unordered");
pub const GAMES_CONFIG_BOARD: Icon = bundled!("games-config-board");
pub const GAMES_HIGHSCORES: Icon = bundled!("games-highscores");
pub const GAMES_HINT: Icon = bundled!("games-hint");
pub const GAMES_SOLVE: Icon = bundled!("games-solve");
pub const GLOBE: Icon = bundled!("globe");
pub const GO_NEXT: Icon = bundled!("go-next");
pub const GO_DOWN: Icon = bundled!("go-down");
pub const GO_HOME: Icon = bundled!("go-home");
pub const HELP_ABOUT: Icon = state!("help-about");
pub const HELP_CONTENTS: Icon = bundled!("help-contents");
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
pub const NETWORK_WIRED: Icon = bundled!("network-wired");
pub const OBJECT_LOCKED: Icon = bundled!("object-locked");
pub const OBJECT_UNLOCKED: Icon = bundled!("object-unlocked");
pub const OFFICE_REPORT: Icon = bundled!("office-report");
pub const OVERFLOW_MENU: Icon = bundled!("overflow-menu");
pub const PACKAGE: Icon = bundled!("package");
pub const PLUGINS: Icon = bundled!("plugins");
pub const PREFERENCES_DESKTOP_THEME_GLOBAL: Icon = bundled!("preferences-desktop-theme-global");
pub const PREFERENCES_SYSTEM_BLUETOOTH: Icon = bundled!("preferences-system-bluetooth");
pub const PREFERENCES_VIRTUALIZATION_CONTAINER: Icon =
    coloured!("preferences-virtualization-container");
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
pub const SYSTEM_SOFTWARE_INSTALL: Icon = coloured!("system-software-install");
pub const SYSTEM_SOFTWARE_UPDATE: Icon = bundled!("system-software-update");
pub const TEXT_X_GENERIC: Icon = bundled!("text-x-generic");
pub const TOOLS: Icon = bundled!("tools");
pub const TOOLS_WIZARD: Icon = bundled!("tools-wizard");
pub const USER_OTHERS: Icon = bundled!("user-others");
pub const UTILITIES_TERMINAL: Icon = bundled!("utilities-terminal");
pub const VCS_DIFF: Icon = bundled!("vcs-diff");
pub const VIEW_CERTIFICATE: Icon = bundled!("view-certificate");
pub const VIEW_LIST_DETAILS: Icon = bundled!("view-list-details");
pub const VIEW_LIST_ICONS: Icon = bundled!("view-list-icons");
pub const VIEW_MEDIA_EQUALIZER: Icon = bundled!("view-media-equalizer");
pub const VIEW_PIM_TASKS: Icon = bundled!("view-pim-tasks");
pub const VIEW_REFRESH: Icon = bundled!("view-refresh");
pub const VIEW_TASK: Icon = bundled!("view-task");
pub const WINDOW_CLOSE: Icon = bundled!("window-close");

pub const ALL: [Icon; 87] = [
    APPLICATIONS_DEVELOPMENT,
    APPLICATIONS_GAMES,
    APPLICATIONS_GRAPHICS,
    APPLICATIONS_INTERNET,
    APPLICATIONS_MULTIMEDIA,
    APPLICATIONS_OFFICE,
    APPLICATIONS_OTHER,
    APPLICATIONS_SYSTEM,
    AUDIO_VOLUME_HIGH,
    BATTERY_GOOD,
    CAMERA_VIDEO,
    CHECKMARK,
    CHRONOMETER,
    COMPUTER,
    CONFIGURE,
    DESKTOP,
    DIALOG_ERROR,
    DIALOG_POSITIVE,
    DIALOG_MESSAGES,
    DIALOG_WARNING,
    DOCUMENTATION,
    DOCUMENT_PROPERTIES,
    DOCUMENT_PREVIEW,
    DOCUMENT_REVERT,
    DRAW_CROSS,
    DRIVE_HARDDISK_ROOT,
    EDIT_CLEAR_ALL,
    EDIT_COPY,
    FORMAT_LIST_UNORDERED,
    GAMES_CONFIG_BOARD,
    GAMES_HIGHSCORES,
    GAMES_HINT,
    GAMES_SOLVE,
    GLOBE,
    GO_NEXT,
    GO_DOWN,
    GO_HOME,
    HELP_ABOUT,
    HELP_CONTENTS,
    INPUT_GAMEPAD,
    INPUT_KEYBOARD,
    INPUT_TOUCHPAD,
    INTERNET_MAIL,
    INTERNET_WEB_BROWSER,
    LANGUAGES,
    MEDIA_PLAYBACK_START,
    MEDIA_PLAYBACK_STOP,
    MEDIA_PLAYLIST_REPEAT,
    NETWORK_CONNECT,
    NETWORK_WIRED,
    OBJECT_LOCKED,
    OBJECT_UNLOCKED,
    OFFICE_REPORT,
    OVERFLOW_MENU,
    PACKAGE,
    PLUGINS,
    PREFERENCES_DESKTOP_THEME_GLOBAL,
    PREFERENCES_SYSTEM_BLUETOOTH,
    PREFERENCES_VIRTUALIZATION_CONTAINER,
    PRINTER,
    REPOSITORY,
    RUN_BUILD,
    RUN_BUILD_CONFIGURE,
    RUN_INSTALL,
    SECURITY_HIGH,
    SIDEBAR_COLLAPSE_LEFT,
    SIDEBAR_EXPAND_LEFT,
    SPEEDOMETER,
    SYSTEM_REBOOT,
    SYSTEM_RUN,
    SYSTEM_SEARCH,
    SYSTEM_SOFTWARE_INSTALL,
    SYSTEM_SOFTWARE_UPDATE,
    TEXT_X_GENERIC,
    TOOLS,
    TOOLS_WIZARD,
    USER_OTHERS,
    UTILITIES_TERMINAL,
    VCS_DIFF,
    VIEW_CERTIFICATE,
    VIEW_LIST_DETAILS,
    VIEW_LIST_ICONS,
    VIEW_MEDIA_EQUALIZER,
    VIEW_PIM_TASKS,
    VIEW_REFRESH,
    VIEW_TASK,
    WINDOW_CLOSE,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    name: &'static str,
    bundled: &'static [u8],
    /// The bundled file is a glyph in one colour, drawn in the text colour.
    tinted: bool,
    themed: bool,
}

/// The icon files a desktop theme has for the app's icons, read off the
/// UI thread.
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

    /// The name to look up in the desktop theme, `None` for an icon the
    /// theme never draws. A glyph asks for the `-symbolic` variant only,
    /// because a theme's plain file of the same name is often coloured.
    pub fn theme_name(&self) -> Option<String> {
        match (self.themed, self.tinted) {
            (false, _) => None,
            (true, true) => Some(format!("{}-symbolic", self.name)),
            (true, false) => Some(self.name.to_owned()),
        }
    }

    /// The theme's file as it is, or the bundled glyph in `tint`. The
    /// caller adds the alt text, see the `UiExt` icon widgets.
    #[must_use = "an image does nothing until it is added to a ui"]
    pub fn image(&self, theme: Option<&ThemeIcons>, tint: Color32) -> Image<'static> {
        let themed = theme.filter(|_| self.themed).and_then(|theme| {
            let file = theme.files.get(self.name)?;
            let image = Image::from_bytes(
                format!("{THEME_URI}{}/{}.svg", theme.theme, self.name),
                Arc::clone(&file.svg),
            );

            Some(if file.tinted { image.tint(tint) } else { image })
        });

        themed.unwrap_or_else(|| {
            let image = Image::from_bytes(
                format!("bytes://icons/bundled/{}.svg", self.name),
                self.bundled,
            );

            if self.tinted { image.tint(tint) } else { image }
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
    /// file in `currentColor` is a glyph too, since unpainted it would draw
    /// black. Icons with their own colours keep them.
    pub fn insert(&mut self, name: &'static str, file: Arc<[u8]>) {
        let text = String::from_utf8_lossy(&file);
        let tinted = named(name).is_some_and(|icon| icon.tinted) || text.contains("currentColor");
        let svg = if tinted {
            Arc::from(mask(&text).into_bytes())
        } else {
            file
        };

        self.files.insert(name, ThemeFile { svg, tinted });
    }

    pub fn theme(&self) -> &str {
        &self.theme
    }
}

/// The app's icon of that freedesktop name, for names that come as data
/// (the catalogs' section icons).
pub fn named(name: &str) -> Option<Icon> {
    ALL.into_iter().find(|icon| icon.name == name)
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
