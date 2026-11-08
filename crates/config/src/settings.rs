//! The user's settings file, `settings.toml`:
//!
//! ```toml
//! [general]
//! language = "de"        # missing: follow the system
//! log_level = "info"
//!
//! [diagnostics]
//! copy_to_clipboard = false
//!
//! [services]
//! show_user_units = false
//!
//! [packages]
//! show_aur = false       # the AUR chip and its daily download
//!
//! [status]
//! refresh = "1h"         # off, or 5m to 24h, such as 45m or 3h
//!
//! [colours]
//! dark_scheme = "nord"   # missing: the desktop's colours
//! light_scheme = "catppuccin-latte"
//! ```
//!
//! Keys the app no longer has, such as an older version's `[appearance]`
//! blur, are skipped on load and dropped at the next save.

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toolbox_plan::SchemeId;

use crate::file::write_atomic;
use crate::refresh::RefreshInterval;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub general: General,
    pub diagnostics: Diagnostics,
    pub services: Services,
    pub packages: Packages,
    pub status: Status,
    pub colours: Colours,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    /// `None` follows the system locale.
    pub language: Option<Language>,
    pub log_level: LogLevel,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Diagnostics {
    pub copy_to_clipboard: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Services {
    pub show_user_units: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Packages {
    /// Off by default: the list downloads from the AUR, whose packages are community scripts.
    pub show_aur: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Status {
    pub refresh: RefreshInterval,
}

/// The window's Theme setting picks which one shows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Colours {
    /// `None` takes the desktop's dark colours.
    pub dark_scheme: Option<SchemeId>,
    /// `None` takes the desktop's light colours.
    pub light_scheme: Option<SchemeId>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::level_filters::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Self::ERROR,
            LogLevel::Warn => Self::WARN,
            LogLevel::Info => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Trace => Self::TRACE,
        }
    }
}

/// Read from the settings file, so it is checked strictly.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Language(String);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a language code such as `de` or `pt_BR`")]
pub struct InvalidLanguage(String);

impl Language {
    /// # Errors
    ///
    /// [`InvalidLanguage`] unless `code` is two or three letters with an
    /// optional `-` or `_` part of two to eight letters or digits.
    pub fn new(code: impl Into<String>) -> Result<Self, InvalidLanguage> {
        let code = code.into();

        if is_language(&code) {
            Ok(Self(code))
        } else {
            Err(InvalidLanguage(code))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Language {
    type Error = InvalidLanguage;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Language> for String {
    fn from(language: Language) -> Self {
        language.0
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot understand {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("cannot encode the settings: {0}")]
    Encode(#[from] toml::ser::Error),
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug)]
pub struct Loaded {
    pub settings: Settings,
    pub problem: Option<SettingsError>,
}

impl Settings {
    /// A missing file is the normal first start. A broken file gives the defaults and the reason.
    /// A file with single bad fields keeps its valid fields and reports the first bad one.
    pub fn load(path: &Path) -> Loaded {
        match read(path) {
            Ok((settings, problem)) => {
                if let Some(problem) = &problem {
                    tracing::warn!("using partly default settings: {problem}");
                }

                Loaded {
                    settings: settings.unwrap_or_default(),
                    problem,
                }
            }
            Err(problem) => {
                tracing::warn!("using default settings: {problem}");
                Loaded {
                    settings: Self::default(),
                    problem: Some(problem),
                }
            }
        }
    }

    /// # Errors
    ///
    /// [`SettingsError`] when the folder or the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        let text = toml::to_string_pretty(self)?;

        write_atomic(path, text.as_bytes()).map_err(|source| SettingsError::Write {
            path: path.to_owned(),
            source,
        })?;
        tracing::debug!("settings saved to {}", path.display());
        Ok(())
    }
}

/// `Ok((None, None))` when the file does not exist. A file with single
/// bad fields gives the valid fields with the first error beside them.
fn read(path: &Path) -> Result<(Option<Settings>, Option<SettingsError>), SettingsError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((None, None)),
        Err(source) => {
            return Err(SettingsError::Read {
                path: path.to_owned(),
                source,
            });
        }
    };

    match toml::from_str::<Settings>(&text) {
        Ok(settings) => Ok((Some(settings), None)),
        Err(source) => {
            let problem = SettingsError::Parse {
                path: path.to_owned(),
                source,
            };

            match lossy(&text) {
                Some(settings) => Ok((Some(settings), Some(problem))),
                None => Err(problem),
            }
        }
    }
}

/// `None` when the text is no TOML table at all.
///
/// Each field is tested alone, so serde's rules apply and a new field needs no line here.
/// Fields absent from the serialised defaults, such as an unset `language`, are read from `text`.
fn lossy(text: &str) -> Option<Settings> {
    let given: toml::Table = toml::from_str(text).ok()?;
    let mut merged = toml::Value::try_from(Settings::default()).ok()?;

    for (name, section) in &given {
        let Some(fields) = section.as_table() else {
            continue;
        };

        let Some(known) = merged.get_mut(name).and_then(toml::Value::as_table_mut) else {
            continue;
        };

        for (key, value) in fields {
            if parses_alone(name, key, value) {
                known.insert(key.clone(), value.clone());
            }
        }
    }

    merged.try_into().ok()
}

fn parses_alone(section: &str, key: &str, value: &toml::Value) -> bool {
    let mut one = toml::Table::new();
    let mut fields = toml::Table::new();

    fields.insert(key.to_owned(), value.clone());
    one.insert(section.to_owned(), toml::Value::Table(fields));

    toml::Value::Table(one).try_into::<Settings>().is_ok()
}

/// Two or three letters with one optional `-` or `_` part of two to
/// eight letters or digits, such as `de` or `pt_BR`.
fn is_language(code: &str) -> bool {
    let (primary, sub) = match code.split_once(['-', '_']) {
        Some((_, rest)) if rest.contains(['-', '_']) => return false,
        Some((primary, rest)) => (primary, Some(rest)),
        None => (code, None),
    };

    if !(2..=3).contains(&primary.len())
        || !primary
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return false;
    }

    sub.is_none_or(|sub| {
        (2..=8).contains(&sub.len())
            && sub
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
    })
}
