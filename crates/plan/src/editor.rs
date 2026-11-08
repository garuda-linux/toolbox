//! The terminal editors the Config Files page may start, and the private
//! copy they edit. `$VISUAL` and `$EDITOR` are command lines in other
//! programs. Here they only pick one of a closed list, so nothing from the
//! environment is ever run.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::component::has_plain_components;
use crate::procedure::{Command, Program};
use crate::shown::Shown;

/// The two folders every [`EditCopy`] lies in, below the user's runtime or
/// cache folder.
pub const EDIT_FOLDER: &str = "garuda-toolbox/edit";

/// Linux's `PATH_MAX`.
const MAX_PATH_BYTES: usize = 4096;

/// A terminal editor that runs inside the Console.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Editor {
    Micro,
    Nano,
    Vim,
    Neovim,
    Vi,
    Helix,
    Emacs,
    Ne,
    Joe,
    Mcedit,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidEditCopy {
    #[error("the path of the copy is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the path of the copy `{}` is not absolute", Shown::new(.0))]
    Relative(String),
    #[error("the path of the copy contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the path of the copy `{}` has an empty, `.` or `..` component", Shown::new(.0))]
    InvalidComponent(String),
    #[error("the copy `{}` is not a file in the toolbox's edit folder", Shown::new(.0))]
    OutsideEditFolder(String),
}

/// The file an editor works on: `<base>/garuda-toolbox/edit/<name>`,
/// absolute and without `..`. Never the config file itself, so no editor
/// needs root.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditCopy(String);

validated_string!(EditCopy, InvalidEditCopy);

impl Editor {
    /// The fallbacks, in the order they are tried when `$VISUAL` and
    /// `$EDITOR` name none of [`Editor::ALL`]. micro is what Garuda ships.
    pub const FALLBACKS: [Self; 3] = [Self::Micro, Self::Nano, Self::Vi];

    pub const ALL: [Self; 10] = [
        Self::Micro,
        Self::Nano,
        Self::Vim,
        Self::Neovim,
        Self::Vi,
        Self::Helix,
        Self::Emacs,
        Self::Ne,
        Self::Joe,
        Self::Mcedit,
    ];

    /// The editor a `$VISUAL` or `$EDITOR` value asks for: the file name
    /// of its first word. Arguments and folders in the value are dropped,
    /// the editor always starts from its fixed path.
    pub fn from_variable(value: &str) -> Option<Self> {
        let word = value.split_whitespace().next()?;
        let name = word.rsplit('/').next()?;

        Self::ALL
            .into_iter()
            .find(|editor| editor.names().contains(&name))
    }

    pub const fn names(self) -> &'static [&'static str] {
        match self {
            Self::Micro => &["micro"],
            Self::Nano => &["nano"],
            Self::Vim => &["vim"],
            Self::Neovim => &["nvim"],
            Self::Vi => &["vi"],
            Self::Helix => &["helix", "hx"],
            Self::Emacs => &["emacs"],
            Self::Ne => &["ne"],
            Self::Joe => &["joe"],
            Self::Mcedit => &["mcedit"],
        }
    }

    pub const fn program(self) -> Program {
        match self {
            Self::Micro => Program::Micro,
            Self::Nano => Program::Nano,
            Self::Vim => Program::Vim,
            Self::Neovim => Program::Neovim,
            Self::Vi => Program::Vi,
            Self::Helix => Program::Helix,
            Self::Emacs => Program::Emacs,
            Self::Ne => Program::Ne,
            Self::Joe => Program::Joe,
            Self::Mcedit => Program::Mcedit,
        }
    }

    /// The editor opened on `copy`. The path is absolute, so no editor can
    /// read it as an option.
    pub fn command(self, copy: &EditCopy) -> Command {
        let command = Command::new(self.program());

        match self {
            // Without it Emacs opens a window of its own outside the Console.
            Self::Emacs => command.arg("-nw"),
            Self::Micro
            | Self::Nano
            | Self::Vim
            | Self::Neovim
            | Self::Vi
            | Self::Helix
            | Self::Ne
            | Self::Joe
            | Self::Mcedit => command,
        }
        .arg(copy.as_str())
    }
}

impl EditCopy {
    /// # Errors
    ///
    /// [`InvalidEditCopy`] for a path that is overlong, relative, has a
    /// control character or an empty, `.` or `..` component, or is no
    /// file directly in a `garuda-toolbox/edit` folder.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidEditCopy> {
        let value = value.into();

        if value.len() > MAX_PATH_BYTES {
            return Err(InvalidEditCopy::TooLong {
                max: MAX_PATH_BYTES,
            });
        }

        // Before anything that quotes the value: an escape sequence never
        // reaches a message.
        if let Some(character) = value.chars().find(|character| character.is_control()) {
            return Err(InvalidEditCopy::InvalidCharacter(character));
        }

        let Some(relative) = value.strip_prefix('/') else {
            return Err(InvalidEditCopy::Relative(value));
        };

        if !has_plain_components(relative) {
            return Err(InvalidEditCopy::InvalidComponent(value));
        }

        let in_edit_folder = value
            .rsplit_once('/')
            .is_some_and(|(folder, _)| folder.ends_with(&format!("/{EDIT_FOLDER}")));

        if !in_edit_folder {
            return Err(InvalidEditCopy::OutsideEditFolder(value));
        }

        Ok(Self(value))
    }
}
