//! The value objects of a user config edit: a file below the home and the
//! words that name an INI group, an INI key, its values or a JSON key.

#[cfg(test)]
mod tests;

use crate::component::has_plain_components;
use crate::shown::Shown;

/// Deeper than any config file below a home.
pub const MAX_PATH_BYTES: usize = 1024;

/// Longer than any group, key or value a toggle needs.
pub const MAX_WORD_BYTES: usize = 255;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidHomePath {
    #[error("the home path is empty")]
    Empty,
    #[error("the home path is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the home path `{}` is absolute, it must be relative to the home", Shown::new(.0))]
    Absolute(String),
    #[error("the home path contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the home path `{}` has an empty, `.` or `..` component", Shown::new(.0))]
    InvalidComponent(String),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidConfigWord {
    #[error("the config word is empty")]
    Empty,
    #[error("the config word is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the config word contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the config word `{}` starts or ends with a blank", Shown::new(.0))]
    Padded(String),
}

/// A file below the user's home, such as `.config/kwinrc`. Relative, so
/// it cannot name a file elsewhere, and without `..`, so it cannot climb
/// out of the home.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HomePath(String);

validated_string!(HomePath, InvalidHomePath, no_display);

/// One INI group, key or value, or a JSON key. A single printable line
/// without the characters that structure an INI file, so writing it can
/// never start a new group or entry.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigWord(String);

validated_string!(ConfigWord, InvalidConfigWord);

impl HomePath {
    /// # Errors
    ///
    /// [`InvalidHomePath`] for an empty, overlong or absolute path, one with
    /// a control character, or one with an empty, `.` or `..` component.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidHomePath> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidHomePath::Empty);
        }

        if value.len() > MAX_PATH_BYTES {
            return Err(InvalidHomePath::TooLong {
                max: MAX_PATH_BYTES,
            });
        }

        // Before anything that quotes the value: an escape sequence never
        // reaches a message.
        if let Some(character) = value.chars().find(|character| character.is_control()) {
            return Err(InvalidHomePath::InvalidCharacter(character));
        }

        if value.starts_with('/') {
            return Err(InvalidHomePath::Absolute(value));
        }

        if !has_plain_components(&value) {
            return Err(InvalidHomePath::InvalidComponent(value));
        }

        Ok(Self(value))
    }
}

impl ConfigWord {
    /// # Errors
    ///
    /// [`InvalidConfigWord`] for an empty or overlong word, one with a
    /// control character, `[`, `]`, `=` or `#`, or blanks around it.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConfigWord> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidConfigWord::Empty);
        }

        if value.len() > MAX_WORD_BYTES {
            return Err(InvalidConfigWord::TooLong {
                max: MAX_WORD_BYTES,
            });
        }

        if let Some(character) = value
            .chars()
            .find(|character| character.is_control() || "[]=#".contains(*character))
        {
            return Err(InvalidConfigWord::InvalidCharacter(character));
        }

        if value.trim() != value {
            return Err(InvalidConfigWord::Padded(value));
        }

        Ok(Self(value))
    }
}

/// As the user knows the file: `~/.config/kwinrc`.
impl std::fmt::Display for HomePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "~/{}", self.0)
    }
}
