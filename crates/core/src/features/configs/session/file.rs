//! Which file an edit is of, and the write that ends it. A root-owned
//! file goes back through the helper, a file of the user is written by
//! the app as the user.

use std::fmt;

use toolbox_plan::{
    ConfigFilePath, ConfigText, ConfigWrite, Fingerprint, HOME_CONFIG_BACKUPS, HomePath, HomeWrite,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditedFile {
    Root(ConfigFilePath),
    Home(HomePath),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditWrite {
    Root(ConfigWrite),
    Home(HomeWrite),
}

impl EditedFile {
    pub fn backup(&self) -> String {
        match self {
            Self::Root(path) => format!("/{}", path.backup()),
            Self::Home(path) => {
                format!("~/.local/state/{HOME_CONFIG_BACKUPS}/{}", path.as_str())
            }
        }
    }

    pub fn write(&self, content: ConfigText, original: Fingerprint) -> EditWrite {
        match self {
            Self::Root(path) => EditWrite::Root(ConfigWrite {
                path: path.clone(),
                content,
                original,
            }),
            Self::Home(path) => EditWrite::Home(HomeWrite {
                path: path.clone(),
                content,
                original,
            }),
        }
    }
}

impl From<ConfigFilePath> for EditedFile {
    fn from(path: ConfigFilePath) -> Self {
        Self::Root(path)
    }
}

impl From<HomePath> for EditedFile {
    fn from(path: HomePath) -> Self {
        Self::Home(path)
    }
}

/// The path as the user reads it: absolute, or below `~/`.
impl fmt::Display for EditedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root(path) => path.fmt(f),
            Self::Home(path) => path.fmt(f),
        }
    }
}
