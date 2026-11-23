//! Colour icons, drawn in their own colours and never tinted.
//!
//! The names are the freedesktop names KDE apps use, looked up in the
//! desktop theme first. The bundled copies in `assets/icons/colour/` have
//! the Breeze dark `currentColor` parts set to the ink `#fcfcfc`.

use super::Icon;

macro_rules! colour {
    ($name:literal) => {
        Icon {
            name: $name,
            theme: $name,
            bundled: include_bytes!(concat!("../../assets/icons/colour/", $name, ".svg")),
            tinted: false,
        }
    };
}

pub const APPLICATIONS_ALL: Icon = colour!("applications-all");
pub const APPLICATIONS_DEVELOPMENT: Icon = colour!("applications-development");
pub const APPLICATIONS_GAMES: Icon = colour!("applications-games");
pub const APPLICATIONS_GRAPHICS: Icon = colour!("applications-graphics");
pub const APPLICATIONS_INTERNET: Icon = colour!("applications-internet");
pub const APPLICATIONS_MULTIMEDIA: Icon = colour!("applications-multimedia");
pub const APPLICATIONS_OFFICE: Icon = colour!("applications-office");
pub const APPLICATIONS_OTHER: Icon = colour!("applications-other");
pub const APPLICATIONS_SYSTEM: Icon = colour!("applications-system");
pub const APPLICATIONS_UTILITIES: Icon = colour!("applications-utilities");
pub const BATTERY_GOOD: Icon = colour!("battery-good");
pub const CAMERA_VIDEO: Icon = colour!("camera-video");
pub const COMPUTER: Icon = colour!("computer");
pub const DESKTOP: Icon = colour!("desktop");
pub const DIALOG_ERROR: Icon = colour!("dialog-error");
pub const DIALOG_INFORMATION: Icon = colour!("dialog-information");
pub const DIALOG_POSITIVE: Icon = colour!("dialog-positive");
pub const DIALOG_WARNING: Icon = colour!("dialog-warning");
pub const DRIVE_HARDDISK: Icon = colour!("drive-harddisk");
pub const DRIVE_HARDDISK_ROOT: Icon = colour!("drive-harddisk-root");
pub const FOLDER_DOWNLOAD: Icon = colour!("folder-download");
pub const HWINFO: Icon = colour!("hwinfo");
pub const INPUT_GAMEPAD: Icon = colour!("input-gamepad");
pub const INPUT_KEYBOARD: Icon = colour!("input-keyboard");
pub const INPUT_TOUCHPAD: Icon = colour!("input-touchpad");
pub const INTERNET_MAIL: Icon = colour!("internet-mail");
pub const INTERNET_WEB_BROWSER: Icon = colour!("internet-web-browser");
pub const NETWORK_SERVER: Icon = colour!("network-server");
pub const NETWORK_WIRED: Icon = colour!("network-wired");
pub const NETWORK_WIRELESS: Icon = colour!("network-wireless");
pub const PACKAGE_X_GENERIC: Icon = colour!("package-x-generic");
pub const PLASMADISCOVER: Icon = colour!("plasmadiscover");
pub const PREFERENCES_DESKTOP_GAMING: Icon = colour!("preferences-desktop-gaming");
pub const PREFERENCES_DESKTOP_LOCALE: Icon = colour!("preferences-desktop-locale");
pub const PREFERENCES_DESKTOP_PERIPHERALS: Icon = colour!("preferences-desktop-peripherals");
pub const PREFERENCES_DESKTOP_THEME_GLOBAL: Icon = colour!("preferences-desktop-theme-global");
pub const PREFERENCES_OTHER: Icon = colour!("preferences-other");
pub const PREFERENCES_DESKTOP_SOUND: Icon = colour!("preferences-desktop-sound");
pub const PREFERENCES_SYSTEM: Icon = colour!("preferences-system");
pub const PREFERENCES_SYSTEM_BLUETOOTH: Icon = colour!("preferences-system-bluetooth");
pub const PREFERENCES_SYSTEM_LINUX: Icon = colour!("preferences-system-linux");
pub const PREFERENCES_SYSTEM_NETWORK: Icon = colour!("preferences-system-network");
pub const PREFERENCES_SYSTEM_SERVICES: Icon = colour!("preferences-system-services");
pub const PREFERENCES_SYSTEM_STARTUP: Icon = colour!("preferences-system-startup");
pub const PREFERENCES_SYSTEM_USERS: Icon = colour!("preferences-system-users");
pub const PREFERENCES_VIRTUALIZATION_CONTAINER: Icon =
    colour!("preferences-virtualization-container");
pub const PRINTER: Icon = colour!("printer");
pub const SECURITY_HIGH: Icon = colour!("security-high");
pub const SYSTEM_RUN: Icon = colour!("system-run");
pub const SYSTEM_SOFTWARE_INSTALL: Icon = colour!("system-software-install");
pub const SYSTEM_SOFTWARE_UPDATE: Icon = colour!("system-software-update");
pub const TEXT_X_GENERIC: Icon = colour!("text-x-generic");
pub const UTILITIES_SYSTEM_MONITOR: Icon = colour!("utilities-system-monitor");
pub const UTILITIES_TERMINAL: Icon = colour!("utilities-terminal");

/// The colour icon a row draws for a symbolic glyph, the Breeze colour
/// file KDE apps use for the same thing. A glyph with none keeps its
/// symbol, as Breeze draws it in one colour only.
pub(super) fn of_row(glyph: Icon) -> Icon {
    match glyph.name() {
        "applications-development" | "run-build" => APPLICATIONS_DEVELOPMENT,
        "applications-games" | "games-config-board" => APPLICATIONS_GAMES,
        "applications-graphics" => APPLICATIONS_GRAPHICS,
        "applications-internet" => APPLICATIONS_INTERNET,
        "applications-multimedia" => APPLICATIONS_MULTIMEDIA,
        "applications-office" => APPLICATIONS_OFFICE,
        "applications-other" => APPLICATIONS_OTHER,
        "applications-system" => APPLICATIONS_SYSTEM,
        "audio-volume-high" => PREFERENCES_DESKTOP_SOUND,
        "battery-good" => BATTERY_GOOD,
        "camera-video" => CAMERA_VIDEO,
        "computer" => COMPUTER,
        "desktop" => DESKTOP,
        "dialog-cancel" | "security-high" => SECURITY_HIGH,
        "dialog-scripts" | "utilities-terminal" => UTILITIES_TERMINAL,
        "drive-harddisk" => DRIVE_HARDDISK,
        "games-hint" | "tools" => APPLICATIONS_UTILITIES,
        "network-wireless" | "network-wireless-signal-good" => NETWORK_WIRELESS,
        "office-report" | "text-x-generic" => TEXT_X_GENERIC,
        "run-install" => SYSTEM_SOFTWARE_INSTALL,
        "speedometer" | "view-task" => UTILITIES_SYSTEM_MONITOR,
        "drive-harddisk-root" => DRIVE_HARDDISK_ROOT,
        "folder-download" => FOLDER_DOWNLOAD,
        "globe" | "internet-web-browser" => INTERNET_WEB_BROWSER,
        "input-gamepad" => INPUT_GAMEPAD,
        "input-keyboard" => INPUT_KEYBOARD,
        "input-touchpad" => INPUT_TOUCHPAD,
        "internet-mail" => INTERNET_MAIL,
        "languages" => PREFERENCES_DESKTOP_LOCALE,
        "network-server" => NETWORK_SERVER,
        "network-wired" => NETWORK_WIRED,
        "package" => PACKAGE_X_GENERIC,
        "preferences-desktop-theme-global" => PREFERENCES_DESKTOP_THEME_GLOBAL,
        "preferences-system-bluetooth" => PREFERENCES_SYSTEM_BLUETOOTH,
        "printer" => PRINTER,
        "system-run" => SYSTEM_RUN,
        "system-software-update" => SYSTEM_SOFTWARE_UPDATE,
        "user-others" => PREFERENCES_SYSTEM_USERS,
        _ => glyph,
    }
}
