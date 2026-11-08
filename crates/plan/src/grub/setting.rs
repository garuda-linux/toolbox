//! The settings the Boot options page may change in `/etc/default/grub`,
//! each with the variable it stands for and the values it gets.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::defaults::{grub_value, set_grub_value, unset_grub_value};
use super::parameters::KernelParameters;
use crate::{Bounded, GrubPicture, GrubThemeFile, MenuEntryId};

/// What `GRUB_DEFAULT` holds when GRUB boots the entry it booted last.
const SAVED: &str = "saved";

/// GRUB's own defaults for unset variables.
const FIRST_ENTRY: &str = "0";
const DEFAULT_TIMEOUT: i64 = 5;

/// Every spelling `grub-mkconfig` takes as "no submenus".
const SUBMENUS_OFF: [&str; 3] = ["y", "yes", "true"];

/// No variable of GRUB: the Garuda theme reads it for its background.
const MENU_PICTURE: &str = "GRUB_MENU_PICTURE";

/// Seconds the menu waits.
pub type GrubTimeout = Bounded<0, 300>;

/// No other line of the file is touched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrubKey {
    Default,
    Timeout,
    KernelParameters,
    Submenus,
    SaveDefault,
    OsProber,
    Background,
    Theme,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultEntry {
    /// The one booted last (needs `GRUB_SAVEDEFAULT`).
    Saved,
    Entry(MenuEntryId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "key", content = "value", rename_all = "kebab-case")]
pub enum GrubSetting {
    Default(DefaultEntry),
    Timeout(GrubTimeout),
    KernelParameters(KernelParameters),
    /// Off writes `GRUB_DISABLE_SUBMENU=y`: every entry in the top menu.
    Submenus(bool),
    SaveDefault(bool),
    /// On writes `GRUB_DISABLE_OS_PROBER=false`.
    OsProber(bool),
    /// `None` comments the variable out.
    Background(Option<GrubPicture>),
    Theme(Option<GrubThemeFile>),
}

/// A list that is empty or sets one variable twice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidGrubSettings {
    #[error("no GRUB setting to change")]
    Empty,
    #[error("{0} is set twice")]
    Repeated(&'static str),
}

/// The settings of one edit of the file: at least one, each variable once.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<GrubSetting>", into = "Vec<GrubSetting>")]
pub struct GrubSettings(Vec<GrubSetting>);

/// `default`, `background`, `menu_picture` and `theme` hold the file's
/// text as written, not a validated value, so they are shown and never
/// passed to a command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrubState {
    /// `GRUB_DEFAULT` as written, `0` when the file sets none.
    pub default: String,
    /// `None` for a value that is no number.
    pub timeout: Option<i64>,
    /// `GRUB_CMDLINE_LINUX_DEFAULT` as written.
    pub command_line: String,
    /// Its words, `None` when one of them is no [`KernelParameter`]
    /// (quotes, a `$`), which the page then leaves alone.
    pub parameters: Option<KernelParameters>,
    pub submenus: bool,
    pub save_default: bool,
    pub os_prober: bool,
    /// `GRUB_BACKGROUND` as written, empty for none.
    pub background: String,
    /// `GRUB_MENU_PICTURE`, which Garuda's GRUB theme reads and which is
    /// all its stock file sets. Empty for none.
    pub menu_picture: String,
    pub theme: String,
}

impl GrubKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Timeout => "timeout",
            Self::KernelParameters => "kernel-parameters",
            Self::Submenus => "submenus",
            Self::SaveDefault => "save-default",
            Self::OsProber => "os-prober",
            Self::Background => "background",
            Self::Theme => "theme",
        }
    }

    pub const fn variable(self) -> &'static str {
        match self {
            Self::Default => "GRUB_DEFAULT",
            Self::Timeout => "GRUB_TIMEOUT",
            Self::KernelParameters => "GRUB_CMDLINE_LINUX_DEFAULT",
            Self::Submenus => "GRUB_DISABLE_SUBMENU",
            Self::SaveDefault => "GRUB_SAVEDEFAULT",
            Self::OsProber => "GRUB_DISABLE_OS_PROBER",
            Self::Background => "GRUB_BACKGROUND",
            Self::Theme => "GRUB_THEME",
        }
    }
}

impl DefaultEntry {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Saved => SAVED,
            Self::Entry(entry) => entry.as_str(),
        }
    }
}

impl GrubSetting {
    pub const fn key(&self) -> GrubKey {
        match self {
            Self::Default(_) => GrubKey::Default,
            Self::Timeout(_) => GrubKey::Timeout,
            Self::KernelParameters(_) => GrubKey::KernelParameters,
            Self::Submenus(_) => GrubKey::Submenus,
            Self::SaveDefault(_) => GrubKey::SaveDefault,
            Self::OsProber(_) => GrubKey::OsProber,
            Self::Background(_) => GrubKey::Background,
            Self::Theme(_) => GrubKey::Theme,
        }
    }

    /// `None` when the variable is commented out instead.
    pub fn value(&self) -> Option<String> {
        let flag = |on: bool| if on { "true" } else { "false" };

        match self {
            Self::Default(entry) => Some(entry.as_str().to_owned()),
            Self::Timeout(seconds) => Some(seconds.to_string()),
            Self::KernelParameters(parameters) => Some(parameters.to_string()),
            Self::Submenus(shown) => Some(if *shown { "n" } else { "y" }.to_owned()),
            Self::SaveDefault(on) => Some(flag(*on).to_owned()),
            Self::OsProber(on) => Some(flag(!*on).to_owned()),
            Self::Background(picture) => picture.as_ref().map(ToString::to_string),
            Self::Theme(theme) => theme.as_ref().map(ToString::to_string),
        }
    }

    /// `config` with this setting, every other byte kept.
    pub fn apply(&self, config: &str) -> String {
        let variable = self.key().variable();

        self.value().map_or_else(
            || unset_grub_value(config, variable),
            |value| set_grub_value(config, variable, &value),
        )
    }

    /// An unset variable counts as GRUB's default for it.
    pub fn is_met_by(&self, state: &GrubState) -> bool {
        match self {
            Self::Default(entry) => state.default == entry.as_str(),
            Self::Timeout(seconds) => state.timeout == Some(i64::from(seconds.get())),
            Self::KernelParameters(parameters) => state.parameters.as_ref() == Some(parameters),
            Self::Submenus(shown) => state.submenus == *shown,
            Self::SaveDefault(on) => state.save_default == *on,
            Self::OsProber(on) => state.os_prober == *on,
            Self::Background(picture) => {
                state.background == picture.as_ref().map_or("", GrubPicture::as_str)
            }
            Self::Theme(theme) => state.theme == theme.as_ref().map_or("", GrubThemeFile::as_str),
        }
    }
}

impl GrubSettings {
    /// # Errors
    ///
    /// [`InvalidGrubSettings`] for no setting or a variable set twice.
    pub fn new(settings: Vec<GrubSetting>) -> Result<Self, InvalidGrubSettings> {
        let mut seen = BTreeSet::new();

        if let Some(twice) = settings.iter().find(|setting| !seen.insert(setting.key())) {
            return Err(InvalidGrubSettings::Repeated(twice.key().variable()));
        }

        if settings.is_empty() {
            return Err(InvalidGrubSettings::Empty);
        }

        Ok(Self(settings))
    }

    pub fn apply(&self, config: &str) -> String {
        self.0
            .iter()
            .fold(config.to_owned(), |text, setting| setting.apply(&text))
    }
}

/// `GRUB_TIMEOUT="10", #GRUB_THEME`: what the Review page shows.
impl fmt::Display for GrubSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, setting) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }

            let variable = setting.key().variable();

            match setting.value() {
                Some(value) => write!(f, "{variable}={}", super::defaults::quote(&value))?,
                None => write!(f, "#{variable}")?,
            }
        }

        Ok(())
    }
}

impl TryFrom<Vec<GrubSetting>> for GrubSettings {
    type Error = InvalidGrubSettings;

    fn try_from(settings: Vec<GrubSetting>) -> Result<Self, Self::Error> {
        Self::new(settings)
    }
}

impl From<GrubSettings> for Vec<GrubSetting> {
    fn from(settings: GrubSettings) -> Self {
        settings.0
    }
}

impl GrubState {
    /// A value only a shell can work out counts as unset.
    pub fn read(config: &str) -> Self {
        let value = |key: GrubKey| grub_value(config, key.variable());
        let command_line = value(GrubKey::KernelParameters).unwrap_or_default();

        Self {
            default: value(GrubKey::Default)
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| FIRST_ENTRY.to_owned()),
            timeout: value(GrubKey::Timeout)
                .map_or(Some(DEFAULT_TIMEOUT), |text| text.trim().parse().ok()),
            parameters: KernelParameters::parse(&command_line),
            command_line,
            submenus: !value(GrubKey::Submenus)
                .is_some_and(|text| SUBMENUS_OFF.contains(&text.as_str())),
            save_default: value(GrubKey::SaveDefault).as_deref() == Some("true"),
            os_prober: value(GrubKey::OsProber).as_deref() == Some("false"),
            background: value(GrubKey::Background).unwrap_or_default(),
            menu_picture: grub_value(config, MENU_PICTURE).unwrap_or_default(),
            theme: value(GrubKey::Theme).unwrap_or_default(),
        }
    }

    /// The picture behind the boot menu: the one `GRUB_BACKGROUND` names,
    /// else the one of the theme.
    pub fn shown_background(&self) -> &str {
        if self.background.is_empty() {
            &self.menu_picture
        } else {
            &self.background
        }
    }

    /// Whether [`Self::shown_background`] is the theme's picture, which no
    /// change of `GRUB_BACKGROUND` takes away.
    pub const fn background_is_from_theme(&self) -> bool {
        self.background.is_empty() && !self.menu_picture.is_empty()
    }
}
