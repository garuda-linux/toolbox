#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{LoaderDefault, Toggle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoaderTimeout {
    Seconds(u32),
    MenuForce,
    MenuHidden,
    MenuDisabled,
}

impl fmt::Display for LoaderTimeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seconds(seconds) => write!(f, "{seconds}"),
            Self::MenuForce => f.write_str("menu-force"),
            Self::MenuHidden => f.write_str("menu-hidden"),
            Self::MenuDisabled => f.write_str("menu-disabled"),
        }
    }
}

/// `None` leaves systemd-boot's own default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoaderSettings {
    pub timeout: Option<LoaderTimeout>,
    pub default: Option<LoaderDefault>,
    #[serde(default, with = "crate::toggle::option_as_bool")]
    pub editor: Option<Toggle>,
    #[serde(default, with = "crate::toggle::option_as_bool")]
    pub auto_entries: Option<Toggle>,
    #[serde(default, with = "crate::toggle::option_as_bool")]
    pub auto_firmware: Option<Toggle>,
}

impl LoaderSettings {
    /// As systemd-boot reads the file: the last line of a setting wins, and
    /// a value it cannot read counts as unset.
    pub fn parse(text: &str) -> Self {
        let mut settings = Self::default();

        for line in text.lines().map(str::trim) {
            if line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once(|character: char| character.is_ascii_whitespace())
                .map_or((line, ""), |(key, value)| (key, value.trim()));

            match key {
                "timeout" => settings.timeout = timeout(value),
                "default" => settings.default = LoaderDefault::new(value).ok(),
                "editor" => settings.editor = switch(value),
                "auto-entries" => settings.auto_entries = switch(value),
                "auto-firmware" => settings.auto_firmware = switch(value),
                _ => {}
            }
        }

        settings
    }
}

fn timeout(value: &str) -> Option<LoaderTimeout> {
    match value {
        "menu-force" => Some(LoaderTimeout::MenuForce),
        "menu-hidden" => Some(LoaderTimeout::MenuHidden),
        "menu-disabled" => Some(LoaderTimeout::MenuDisabled),
        seconds => seconds.parse().ok().map(LoaderTimeout::Seconds),
    }
}

/// The words systemd's `parse_boolean` takes.
fn switch(value: &str) -> Option<Toggle> {
    match value {
        "1" | "yes" | "y" | "true" | "t" | "on" => Some(Toggle::On),
        "0" | "no" | "n" | "false" | "f" | "off" => Some(Toggle::Off),
        _ => None,
    }
}
