//! Where an account lives and which number it gets: the home folder of
//! `useradd -d` and `usermod -d`, and the user id of `useradd -u`. Both
//! land in `/etc/passwd` and in the argv of a program that runs as root.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::component::has_plain_components;

/// `PATH_MAX` is far larger, but no home folder lives that deep.
pub const MAX_HOME_BYTES: usize = 255;

/// `SYS_UID_MAX` and `UID_MAX` of `/etc/login.defs`: system accounts end
/// below the first, accounts of people at the second.
pub const LAST_SYSTEM_USER_ID: u32 = 999;
pub const LAST_USER_ID: u32 = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidHomeDirectory {
    #[error("the home folder is empty")]
    Empty,
    #[error("the home folder is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the home folder is not an absolute path")]
    NotAbsolute,
    #[error("the home folder contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the home folder has an empty, `.` or `..` component")]
    InvalidComponent,
    #[error("the home folder starts or ends with a blank")]
    OuterBlank,
}

impl InvalidHomeDirectory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "home-empty",
            Self::TooLong { max: _ } => "home-too-long",
            Self::NotAbsolute => "home-not-absolute",
            Self::InvalidCharacter(_) => "home-invalid-character",
            Self::InvalidComponent => "home-invalid-component",
            Self::OuterBlank => "home-outer-blank",
        }
    }
}

/// An absolute path below `/` such as `/home/ada`, without `..`.
///
/// It starts with `/`, so it can never be read as an option. It holds no
/// `:` (the separator of `/etc/passwd`) and no control character, and
/// every part between two `/` is a plain name, so the path names one
/// folder and nothing above it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HomeDirectory(String);

validated_string!(HomeDirectory, InvalidHomeDirectory);

impl HomeDirectory {
    /// # Errors
    ///
    /// [`InvalidHomeDirectory`] for an empty, overlong or relative path,
    /// `/` itself, one with a control character or `:`, with blanks
    /// around it, or with an empty, `.` or `..` component (which covers
    /// `//` and a trailing `/`).
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidHomeDirectory> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidHomeDirectory::Empty);
        }

        if value.len() > MAX_HOME_BYTES {
            return Err(InvalidHomeDirectory::TooLong {
                max: MAX_HOME_BYTES,
            });
        }

        if let Some(character) = value
            .chars()
            .find(|character| character.is_control() || *character == ':')
        {
            return Err(InvalidHomeDirectory::InvalidCharacter(character));
        }

        if value.trim() != value {
            return Err(InvalidHomeDirectory::OuterBlank);
        }

        let Some(rest) = value.strip_prefix('/') else {
            return Err(InvalidHomeDirectory::NotAbsolute);
        };

        if !has_plain_components(rest) {
            return Err(InvalidHomeDirectory::InvalidComponent);
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the user id {0} is not between 1 and {LAST_USER_ID}")]
pub struct InvalidUserId(pub u32);

/// A user id a new account may ask for: never 0, never above `UID_MAX`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(u32);

impl UserId {
    /// # Errors
    ///
    /// [`InvalidUserId`] for 0 and for a number above [`LAST_USER_ID`].
    pub const fn new(value: u32) -> Result<Self, InvalidUserId> {
        if value == 0 || value > LAST_USER_ID {
            Err(InvalidUserId(value))
        } else {
            Ok(Self(value))
        }
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    /// In the range useradd keeps for the accounts of services.
    pub const fn is_system(self) -> bool {
        self.0 <= LAST_SYSTEM_USER_ID
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "path", rename_all = "kebab-case")]
pub enum NewHome {
    /// useradd picks the path (`/home/<name>`) and makes the folder from
    /// `/etc/skel`.
    #[default]
    Standard,
    At(HomeDirectory),
    /// No folder is made. The account still names useradd's default path.
    Absent,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HomeChange {
    pub home: HomeDirectory,
    /// Without it only the account's entry changes.
    pub move_files: bool,
}
