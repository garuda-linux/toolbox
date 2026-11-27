//! The checks of entries that edit a config file in the user's home, and
//! the entry fields that only make sense with them.

use serde::Deserialize;
use toolbox_plan::{ConfigReload, ConfigWord, HomePath, UnitName, UserConfigEdit};

use super::extras;
use crate::{Problem, source};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUserFile {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUserIni {
    path: String,
    group: String,
    key: String,
    on: String,
    off: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUserJson {
    path: String,
    key: String,
}

impl RawUserFile {
    pub fn validate(self) -> Result<UserConfigEdit, Problem> {
        if source::is_blank(&self.content) {
            return Err(Problem::EmptyContent);
        }

        Ok(UserConfigEdit::File {
            path: path(self.path)?,
            content: self.content,
        })
    }
}

impl RawUserIni {
    pub fn validate(self) -> Result<UserConfigEdit, Problem> {
        Ok(UserConfigEdit::IniKey {
            path: path(self.path)?,
            group: word(self.group)?,
            key: word(self.key)?,
            on: word(self.on)?,
            off: word(self.off)?,
        })
    }
}

impl RawUserJson {
    pub fn validate(self) -> Result<UserConfigEdit, Problem> {
        Ok(UserConfigEdit::JsonFlag {
            path: path(self.path)?,
            key: word(self.key)?,
        })
    }
}

/// Units restarted after the edit, which only a user config edit may name.
pub fn restart_units(names: Vec<String>, edits_config: bool) -> Result<Vec<UnitName>, Problem> {
    if names.is_empty() {
        return Ok(Vec::new());
    }

    if !edits_config {
        return Err(Problem::RestartWithoutUserConfig);
    }

    let mut units = Vec::with_capacity(names.len());

    for name in names {
        extras::push_unit(&mut units, name, Problem::RestartRepeats)?;
    }

    Ok(units)
}

/// The program to tell after the edit, only for an edit of a user config.
pub fn reload(name: Option<&str>, edits_config: bool) -> Result<Option<ConfigReload>, Problem> {
    let Some(name) = name else {
        return Ok(None);
    };

    if !edits_config {
        return Err(Problem::ReloadWithoutUserConfig);
    }

    ConfigReload::from_id(name)
        .map(Some)
        .ok_or_else(|| Problem::UnknownReload(name.to_owned()))
}

fn path(value: String) -> Result<HomePath, Problem> {
    HomePath::new(value).map_err(Problem::BadHomePath)
}

fn word(value: String) -> Result<ConfigWord, Problem> {
    ConfigWord::new(value).map_err(Problem::BadConfigWord)
}
