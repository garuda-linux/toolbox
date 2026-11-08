//! The content of a config file on its way between the helper and the
//! app, and the mark that tells whether a file changed meanwhile.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::ConfigFilePath;
use crate::procedure::{Builtin, Invocation};
use crate::shown::Shown;

/// Far above any config file a person edits. A larger file is neither
/// handed out nor written.
pub const MAX_CONFIG_BYTES: usize = 1 << 20;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidConfigText {
    #[error("the file is larger than {max} bytes")]
    TooLarge { max: usize },
    #[error("the file is no text")]
    NotText,
}

/// The text of a config file: UTF-8, at most [`MAX_CONFIG_BYTES`] and
/// without a NUL byte.
///
/// Never the content in debug text: a file may hold keys or tokens, and
/// debug text ends up in logs.
#[derive(Clone, PartialEq, Eq)]
pub struct ConfigText(String);

validated_string!(ConfigText, InvalidConfigText, redacted);

/// Length and FNV-1a hash of a file's bytes.
///
/// The app sends the mark of the version the user edited, and the helper
/// replaces the file only while it still has that mark. It guards against
/// a second writer, not against an attacker: whoever can write the file
/// as root needs no collision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint {
    bytes: u64,
    hash: u64,
}

/// Decoding refuses a path that is not [`ConfigFilePath::is_writable`], so no
/// request reaches the helper's write with another file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawConfigWrite")]
pub struct ConfigWrite {
    pub path: ConfigFilePath,
    pub content: ConfigText,
    pub original: Fingerprint,
}

#[derive(Deserialize)]
struct RawConfigWrite {
    path: ConfigFilePath,
    content: ConfigText,
    original: Fingerprint,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("`{}` is never written as a config file", Shown::new(.0.as_str()))]
pub struct NotWritable(ConfigFilePath);

impl TryFrom<RawConfigWrite> for ConfigWrite {
    type Error = NotWritable;

    fn try_from(raw: RawConfigWrite) -> Result<Self, Self::Error> {
        if !raw.path.is_writable() {
            return Err(NotWritable(raw.path));
        }

        Ok(Self {
            path: raw.path,
            content: raw.content,
            original: raw.original,
        })
    }
}

impl ConfigWrite {
    pub fn invocation(&self) -> Invocation {
        Invocation::Builtin(Builtin::WriteConfigFile(self.clone()))
    }
}

impl fmt::Display for ConfigWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "write {} ({} bytes), the old one kept as /{}  # only when it did not change meanwhile",
            self.path,
            self.content.as_str().len(),
            self.path.backup()
        )
    }
}

impl ConfigText {
    /// # Errors
    ///
    /// [`InvalidConfigText`] for a text that is too large or holds a NUL.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConfigText> {
        let value = value.into();

        if value.len() > MAX_CONFIG_BYTES {
            return Err(InvalidConfigText::TooLarge {
                max: MAX_CONFIG_BYTES,
            });
        }

        if value.contains('\0') {
            return Err(InvalidConfigText::NotText);
        }

        Ok(Self(value))
    }

    /// # Errors
    ///
    /// [`InvalidConfigText`] for bytes that are too many, no UTF-8 or
    /// hold a NUL.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, InvalidConfigText> {
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(InvalidConfigText::TooLarge {
                max: MAX_CONFIG_BYTES,
            });
        }

        String::from_utf8(bytes)
            .map_err(|_not_utf8| InvalidConfigText::NotText)
            .and_then(Self::new)
    }

    pub fn fingerprint(&self) -> Fingerprint {
        Fingerprint::of(self.0.as_bytes())
    }
}

impl Fingerprint {
    pub fn of(bytes: &[u8]) -> Self {
        let hash = bytes.iter().fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        });

        Self {
            bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            hash,
        }
    }
}
