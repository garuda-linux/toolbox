//! Edits of config files in the user's home, switched on and off from the
//! System Settings page. The app runs them as the user, never as root.
//! The text transformations live here, without I/O, so the app's probe and
//! the step that writes agree on what "on" means.

mod ini;
mod json;
mod path;
mod reload;
#[cfg(test)]
mod tests;

use std::fmt;

pub use path::{ConfigWord, HomePath, InvalidConfigWord, InvalidHomePath};
pub use reload::ConfigReload;
use serde::{Deserialize, Serialize};

use crate::Toggle;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum UserConfigEdit {
    /// On: the file exists. Turning on writes `content`, off removes it.
    File { path: HomePath, content: String },
    /// On: `key` in `[group]` has the value `on`. Off writes `off`, but only
    /// over an existing entry: a missing key already means the default.
    IniKey {
        path: HomePath,
        group: ConfigWord,
        key: ConfigWord,
        on: ConfigWord,
        off: ConfigWord,
    },
    /// On: the top-level `key` of a JSON object is `true`. Off removes it.
    JsonFlag { path: HomePath, key: ConfigWord },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rewrite {
    Write(String),
    Remove,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EditError {
    #[error("the file is not UTF-8 text")]
    NotText,
    #[error("the file is not valid JSON ({0})")]
    InvalidJson(String),
    #[error("the file holds JSON, but not an object")]
    NotJsonObject,
}

impl UserConfigEdit {
    pub const fn path(&self) -> &HomePath {
        match self {
            Self::File { path, content: _ }
            | Self::IniKey {
                path,
                group: _,
                key: _,
                on: _,
                off: _,
            }
            | Self::JsonFlag { path, key: _ } => path,
        }
    }

    /// Whether the edit is on, given the file's bytes (`None` when it is
    /// missing). A file the edit cannot read counts as off.
    pub fn is_on(&self, current: Option<&[u8]>) -> bool {
        let text = || std::str::from_utf8(current?).ok();

        match self {
            Self::File {
                path: _,
                content: _,
            } => current.is_some(),
            Self::IniKey {
                path: _,
                group,
                key,
                on,
                off: _,
            } => {
                text().and_then(|text| ini::value(text, group.as_str(), key.as_str()))
                    == Some(on.as_str())
            }
            Self::JsonFlag { path: _, key } => {
                text().is_some_and(|text| json::is_on(text, key.as_str()))
            }
        }
    }

    /// What to do to the file (`current`, `None` when missing) to turn the
    /// edit on or off.
    ///
    /// # Errors
    ///
    /// [`EditError`] for an INI or JSON file that is not text, or a JSON
    /// file that holds no object.
    pub fn rewrite(&self, current: Option<&[u8]>, wish: Toggle) -> Result<Rewrite, EditError> {
        let on = wish.is_on();
        match self {
            Self::File { path: _, content } => Ok(match (current, on) {
                (Some(bytes), true) if bytes == content.as_bytes() => Rewrite::Unchanged,
                (Some(_) | None, true) => Rewrite::Write(content.clone()),
                (Some(_), false) => Rewrite::Remove,
                (None, false) => Rewrite::Unchanged,
            }),
            Self::IniKey {
                path: _,
                group,
                key,
                on: on_value,
                off,
            } => {
                let text = text_of(current)?;
                let present = ini::value(text, group.as_str(), key.as_str()).is_some();
                let value = if on { on_value } else { off };

                if !on && !present {
                    return Ok(Rewrite::Unchanged);
                }

                Ok(changed(
                    text,
                    ini::set(text, group.as_str(), key.as_str(), value.as_str()),
                ))
            }
            Self::JsonFlag { path: _, key } => {
                Ok(json::set(text_of(current)?, key.as_str(), wish)?
                    .map_or(Rewrite::Unchanged, Rewrite::Write))
            }
        }
    }

    pub fn describe(&self, wish: Toggle) -> String {
        let on = wish.is_on();
        match self {
            Self::File { path, content: _ } => {
                if on {
                    format!("write {path}")
                } else {
                    format!("rm -f {path}")
                }
            }
            Self::IniKey {
                path,
                group,
                key,
                on: on_value,
                off,
            } => {
                let value = if on { on_value } else { off };

                format!("set {key}={value} in [{group}] of {path}")
            }
            Self::JsonFlag { path, key } => {
                if on {
                    format!("set \"{key}\": true in {path}")
                } else {
                    format!("remove \"{key}\" from {path}")
                }
            }
        }
    }
}

impl fmt::Display for UserConfigEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File { path, content: _ } => path.fmt(f),
            Self::IniKey {
                path,
                group,
                key,
                on: _,
                off: _,
            } => write!(f, "{path} [{group}] {key}"),
            Self::JsonFlag { path, key } => write!(f, "{path} {key}"),
        }
    }
}

/// A missing file reads as empty, so a first edit creates it.
fn text_of(current: Option<&[u8]>) -> Result<&str, EditError> {
    current.map_or(Ok(""), |bytes| {
        std::str::from_utf8(bytes).map_err(|_error| EditError::NotText)
    })
}

fn changed(before: &str, after: String) -> Rewrite {
    if before == after {
        Rewrite::Unchanged
    } else {
        Rewrite::Write(after)
    }
}
