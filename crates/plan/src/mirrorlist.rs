//! A pacman mirrorlist the helper may write as root. pacman reads more than
//! servers from such a file (an `Include` could pull in any file), so only
//! comments and plain https `Server` lines pass.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The Chaotic-AUR list is about 4 KiB. Anything near this is not one.
pub const MAX_MIRRORLIST_BYTES: usize = 64 * 1024;

const SERVER_PREFIX: &str = "Server = https://";

/// The only variables pacman expands in a server URL.
const PLACEHOLDERS: [&str; 2] = ["$repo", "$arch"];

/// Why a text is not a valid [`Mirrorlist`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidMirrorlist {
    #[error("the mirrorlist is larger than {max} bytes")]
    TooLarge { max: usize },
    #[error("the mirrorlist has no Server line")]
    NoServer,
    #[error("line {line} of the mirrorlist is neither a comment nor an https Server line")]
    InvalidLine { line: usize },
}

/// Text of comments and `Server = https://<host>/<path>` lines, at least
/// one of them a server.
#[derive(Clone, PartialEq, Eq)]
pub struct Mirrorlist {
    text: String,
    servers: usize,
}

impl Mirrorlist {
    /// # Errors
    ///
    /// [`InvalidMirrorlist`] for a text that is too large, has no server or
    /// holds a line that is neither a comment nor an https server.
    ///
    /// # Examples
    ///
    /// ```
    /// use toolbox_plan::Mirrorlist;
    ///
    /// let mirrorlist = Mirrorlist::new("Server = https://cdn.example/$repo/$arch\n").unwrap();
    ///
    /// assert_eq!(mirrorlist.servers(), 1);
    /// ```
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidMirrorlist> {
        let text = text.into();

        if text.len() > MAX_MIRRORLIST_BYTES {
            return Err(InvalidMirrorlist::TooLarge {
                max: MAX_MIRRORLIST_BYTES,
            });
        }

        let mut servers = 0_usize;

        for (number, raw) in (1_usize..).zip(text.split('\n')) {
            let line = raw.strip_suffix('\r').unwrap_or(raw);

            if line.trim().is_empty() {
                continue;
            }

            if line.chars().any(char::is_control) {
                return Err(InvalidMirrorlist::InvalidLine { line: number });
            }

            if line.starts_with('#') {
                continue;
            }

            if !is_server(line) {
                return Err(InvalidMirrorlist::InvalidLine { line: number });
            }

            servers += 1;
        }

        if servers == 0 {
            return Err(InvalidMirrorlist::NoServer);
        }

        Ok(Self { text, servers })
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub const fn servers(&self) -> usize {
        self.servers
    }
}

impl fmt::Debug for Mirrorlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mirrorlist")
            .field("servers", &self.servers)
            .field("bytes", &self.text.len())
            .finish()
    }
}

impl Serialize for Mirrorlist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for Mirrorlist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Self::new(text).map_err(serde::de::Error::custom)
    }
}

fn is_server(line: &str) -> bool {
    let Some((host, path)) = line
        .strip_prefix(SERVER_PREFIX)
        .and_then(|url| url.split_once('/'))
    else {
        return false;
    };

    is_hostname(host) && is_path(path)
}

/// RFC 1123 host names. No port, no user info, no IP literal in brackets.
fn is_hostname(host: &str) -> bool {
    (1..=253).contains(&host.len())
        && host.split('.').all(|label| {
            (1..=63).contains(&label.len())
                && label
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
}

/// No whitespace or quotes, and `$` only as a placeholder pacman expands.
fn is_path(path: &str) -> bool {
    let plain = |part: &str| {
        part.chars().all(|character| {
            !character.is_whitespace() && !character.is_control() && !"'\"`$".contains(character)
        })
    };

    let mut parts = path.split('$');
    let first_is_plain = parts.next().is_some_and(plain);

    first_is_plain
        && parts.all(|part| {
            PLACEHOLDERS.iter().any(|placeholder| {
                placeholder
                    .strip_prefix('$')
                    .and_then(|name| part.strip_prefix(name))
                    .is_some_and(plain)
            })
        })
}
