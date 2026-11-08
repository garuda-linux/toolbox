//! A login shell the helper may set with `chsh`. The path lands in
//! `/etc/passwd` and in `chsh`'s argv, so it must be a plain absolute path:
//! no field separator, no whitespace, no way to climb out of a folder.

#[cfg(test)]
mod tests;

use crate::component::has_plain_components;
use crate::shown::Shown;

/// `PATH_MAX` is far larger, but no shell lives that deep.
pub const MAX_SHELL_PATH_BYTES: usize = 255;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidShellPath {
    #[error("the shell path is empty")]
    Empty,
    #[error("the shell path is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the shell path `{}` is not absolute", Shown::new(.0))]
    NotAbsolute(String),
    #[error("the shell path contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the shell path `{}` has an empty, `.` or `..` component", Shown::new(.0))]
    InvalidComponent(String),
}

/// An absolute path such as `/usr/bin/zsh`, printable, without `..`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShellPath(String);

validated_string!(ShellPath, InvalidShellPath);

impl ShellPath {
    /// # Errors
    ///
    /// [`InvalidShellPath`] for an empty, overlong or relative path, one
    /// with whitespace, a control character or `:`, or one with an empty,
    /// `.` or `..` component.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidShellPath> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidShellPath::Empty);
        }

        if value.len() > MAX_SHELL_PATH_BYTES {
            return Err(InvalidShellPath::TooLong {
                max: MAX_SHELL_PATH_BYTES,
            });
        }

        // `:` separates the fields of `/etc/passwd`.
        if let Some(character) = value.chars().find(|character| {
            character.is_whitespace() || character.is_control() || *character == ':'
        }) {
            return Err(InvalidShellPath::InvalidCharacter(character));
        }

        let Some(rest) = value.strip_prefix('/') else {
            return Err(InvalidShellPath::NotAbsolute(value));
        };

        if !has_plain_components(rest) {
            return Err(InvalidShellPath::InvalidComponent(value));
        }

        Ok(Self(value))
    }

    pub fn program(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// What a list shows for each of `shells`: the program's name, or the
    /// whole path where two paths lead to one name.
    pub fn names(shells: &[Self]) -> Vec<String> {
        shells
            .iter()
            .map(|shell| {
                let program = shell.program();
                let twice = shells
                    .iter()
                    .filter(|other| other.program() == program)
                    .count()
                    > 1;

                if twice { shell.as_str() } else { program }.to_owned()
            })
            .collect()
    }
}
