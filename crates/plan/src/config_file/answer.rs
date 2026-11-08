//! What `garuda-toolbox-helper fetch <path>` answers: one JSON line.

use serde::{Deserialize, Serialize};

use super::{ConfigFilePath, ConfigText};
use crate::protocol::{DecodeError, decode};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum FetchAnswer {
    Text {
        /// The file the text is of: the path that was asked for, or where
        /// its symlinks lead. A write goes to this file.
        path: ConfigFilePath,
        content: ConfigText,
    },
    Refused {
        reason: FetchRefusal,
    },
}

/// An answer, not a failure of the helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FetchRefusal {
    Missing,
    /// A symlink stands where the real path was a moment ago. Links are
    /// followed once, before the file is opened, and never while it is.
    Link,
    /// A symlink that leads to no file, or in a circle.
    Dangling,
    /// A symlink to a file outside the folders the toolbox edits, or to
    /// one of the files that never leave the helper.
    Outside,
    /// A folder, a FIFO, a device or a socket.
    NotRegular,
    TooLarge,
    /// Not UTF-8, or with a NUL byte.
    NotText,
}

impl FetchAnswer {
    /// Decodes the helper's answer line. The content is validated while
    /// decoding, so an answer with anything else is refused whole.
    ///
    /// # Errors
    ///
    /// [`DecodeError`] for an overlong line or anything but an answer.
    pub fn parse(line: &str) -> Result<Self, DecodeError> {
        decode(line)
    }
}

impl FetchRefusal {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Link => "link",
            Self::Dangling => "dangling",
            Self::Outside => "outside",
            Self::NotRegular => "not-regular",
            Self::TooLarge => "too-large",
            Self::NotText => "not-text",
        }
    }
}
