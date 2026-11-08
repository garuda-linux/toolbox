//! Only the changed settings of `loader.conf` change. Every other line
//! stays byte for byte, such as the ones garuda-common-settings' config
//! agent adds.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{LoaderSettings, LoaderTimeout};
use crate::LoaderDefault;
use crate::lines::push_line;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoaderKey {
    Timeout,
    Default,
    Editor,
    AutoEntries,
    AutoFirmware,
}

/// `None` comments the setting out, so systemd-boot's default applies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "key", content = "value", rename_all = "kebab-case")]
pub enum LoaderSetting {
    Timeout(Option<LoaderTimeout>),
    Default(Option<LoaderDefault>),
    Editor(Option<bool>),
    AutoEntries(Option<bool>),
    AutoFirmware(Option<bool>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidLoaderChange {
    #[error("no loader.conf setting to change")]
    Empty,
    #[error("{0} is set twice")]
    Repeated(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<LoaderSetting>", into = "Vec<LoaderSetting>")]
pub struct LoaderChange(Vec<LoaderSetting>);

impl LoaderKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Default => "default",
            Self::Editor => "editor",
            Self::AutoEntries => "auto-entries",
            Self::AutoFirmware => "auto-firmware",
        }
    }
}

impl LoaderSetting {
    pub const fn loader_key(&self) -> LoaderKey {
        match self {
            Self::Timeout(_) => LoaderKey::Timeout,
            Self::Default(_) => LoaderKey::Default,
            Self::Editor(_) => LoaderKey::Editor,
            Self::AutoEntries(_) => LoaderKey::AutoEntries,
            Self::AutoFirmware(_) => LoaderKey::AutoFirmware,
        }
    }

    pub const fn key(&self) -> &'static str {
        self.loader_key().as_str()
    }

    pub fn is_met_by(&self, settings: &LoaderSettings) -> bool {
        match self {
            Self::Timeout(timeout) => settings.timeout == *timeout,
            Self::Default(default) => settings.default == *default,
            Self::Editor(on) => settings.editor == *on,
            Self::AutoEntries(on) => settings.auto_entries == *on,
            Self::AutoFirmware(on) => settings.auto_firmware == *on,
        }
    }

    fn value(&self) -> Option<String> {
        let switch = |on: bool| if on { "yes" } else { "no" }.to_owned();

        match self {
            Self::Timeout(timeout) => timeout.map(|timeout| timeout.to_string()),
            Self::Default(default) => default.as_ref().map(|default| default.as_str().to_owned()),
            Self::Editor(on) | Self::AutoEntries(on) | Self::AutoFirmware(on) => on.map(switch),
        }
    }

    fn apply(&self, text: &str) -> String {
        let key = self.key();
        let lines: Vec<&str> = text.split_inclusive('\n').collect();

        let Some(value) = self.value() else {
            return lines
                .iter()
                .map(|line| {
                    if names(line, key) {
                        format!("#{line}")
                    } else {
                        (*line).to_owned()
                    }
                })
                .collect();
        };

        let wanted = format!("{key} {value}");
        let found = lines
            .iter()
            .rposition(|line| names(line, key))
            .or_else(|| lines.iter().rposition(|line| commented(line, key)));

        let Some(position) = found else {
            let mut appended = text.to_owned();

            push_line(&mut appended, &wanted);
            return appended;
        };

        lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                if index == position {
                    let ending = line
                        .strip_prefix(line.trim_end_matches(['\r', '\n']))
                        .unwrap_or_default();

                    format!("{wanted}{ending}")
                } else {
                    (*line).to_owned()
                }
            })
            .collect()
    }
}

impl LoaderChange {
    /// # Errors
    ///
    /// [`InvalidLoaderChange`] for no setting or a setting named twice.
    pub fn new(settings: Vec<LoaderSetting>) -> Result<Self, InvalidLoaderChange> {
        let mut seen = BTreeSet::new();

        if let Some(twice) = settings.iter().find(|setting| !seen.insert(setting.key())) {
            return Err(InvalidLoaderChange::Repeated(twice.key()));
        }

        if settings.is_empty() {
            return Err(InvalidLoaderChange::Empty);
        }

        Ok(Self(settings))
    }

    pub fn iter(&self) -> impl Iterator<Item = &LoaderSetting> {
        self.0.iter()
    }

    pub fn apply(&self, text: &str) -> String {
        self.0
            .iter()
            .fold(text.to_owned(), |text, setting| setting.apply(&text))
    }
}

impl fmt::Display for LoaderChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, setting) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }

            match setting.value() {
                Some(value) => write!(f, "{} {value}", setting.key())?,
                None => write!(f, "no {}", setting.key())?,
            }
        }

        Ok(())
    }
}

impl TryFrom<Vec<LoaderSetting>> for LoaderChange {
    type Error = InvalidLoaderChange;

    fn try_from(settings: Vec<LoaderSetting>) -> Result<Self, Self::Error> {
        Self::new(settings)
    }
}

impl From<LoaderChange> for Vec<LoaderSetting> {
    fn from(change: LoaderChange) -> Self {
        change.0
    }
}

fn names(line: &str, key: &str) -> bool {
    first_word(line.trim_start()) == Some(key)
}

fn commented(line: &str, key: &str) -> bool {
    line.trim_start()
        .strip_prefix('#')
        .is_some_and(|rest| first_word(rest.trim_start()) == Some(key))
}

fn first_word(line: &str) -> Option<&str> {
    line.split_whitespace().next()
}
