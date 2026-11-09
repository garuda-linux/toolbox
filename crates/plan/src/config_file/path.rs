//! The path of a root-owned config file.

#[cfg(test)]
mod tests;

use crate::component::has_plain_components;
use crate::shown::Shown;

/// The folders whose files the helper hands out and replaces.
///
/// Exactly what the root-owned entries of `data/catalog/config-files.toml`
/// need, and `cargo xtask catalog-validate` fails for an entry outside of
/// them.
pub const CONFIG_ROOTS: [&str; 4] = [
    "/etc/",
    "/boot/grub/",
    "/usr/share/grub/themes/",
    "/usr/share/icons/default/",
];

/// Files below [`CONFIG_ROOTS`] that never leave the helper, by prefix.
///
/// They hold password hashes, keys or passphrases. A prefix also catches
/// the copies beside a file (`shadow-`, `shadow.pacnew`) and everything in
/// a folder. The Users page changes the hashes through typed actions.
pub const WITHHELD_CONFIG_FILES: [&str; 15] = [
    "/etc/shadow",
    "/etc/gshadow",
    "/etc/security/opasswd",
    "/etc/ssh/ssh_host_",
    "/etc/NetworkManager/system-connections",
    "/etc/wpa_supplicant",
    "/etc/ipsec.secrets",
    "/etc/ipsec.d",
    "/etc/wireguard",
    "/etc/cryptsetup-keys.d",
    "/etc/pacman.d/gnupg",
    "/etc/ppp/chap-secrets",
    "/etc/ppp/pap-secrets",
    "/etc/ppp/srp-secrets",
    "/etc/ssl/private",
];

/// File name endings below [`CONFIG_ROOTS`] that never leave the helper.
///
/// A systemd `.netdev` file holds the `PrivateKey=` of a `WireGuard` device,
/// and it lies in `/etc/systemd/network` beside the files the page edits.
pub const WITHHELD_CONFIG_SUFFIXES: [&str; 1] = [".netdev"];

/// Where the helper keeps the previous version of a file it replaced,
/// relative to `/`. The file's own path follows, so `/etc/fstab` is kept
/// as `/var/lib/garuda-toolbox/backups/etc/fstab`.
pub const CONFIG_BACKUPS: &str = "var/lib/garuda-toolbox/backups";

/// `PATH_MAX` minus room for the backup folder in front of it.
pub const MAX_PATH_BYTES: usize = 1024;

/// The longest file name Linux takes.
const MAX_NAME_BYTES: usize = 255;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidConfigFilePath {
    #[error("the config file path is empty")]
    Empty,
    #[error("the config file path is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the config file path `{}` is not absolute", Shown::new(.0))]
    Relative(String),
    #[error("the config file path contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the config file path `{}` has an empty, overlong, `.` or `..` component", Shown::new(.0))]
    InvalidComponent(String),
    #[error("the config file path `{}` is outside the folders the toolbox edits", Shown::new(.0))]
    OutsideRoots(String),
    #[error("`{}` is never read or written as a config file", Shown::new(.0))]
    Withheld(String),
}

/// An absolute, normalised path below one of [`CONFIG_ROOTS`].
///
/// No `..`, `.` or empty component, no control character, no trailing
/// slash. So it names one file or folder there and nothing a symlink-free
/// walk from `/` could not reach.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigFilePath(String);

validated_string!(ConfigFilePath, InvalidConfigFilePath);

impl ConfigFilePath {
    /// # Errors
    ///
    /// [`InvalidConfigFilePath`] for a path that breaks one of the rules
    /// of the type.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConfigFilePath> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidConfigFilePath::Empty);
        }

        if value.len() > MAX_PATH_BYTES {
            return Err(InvalidConfigFilePath::TooLong {
                max: MAX_PATH_BYTES,
            });
        }

        // Before anything that quotes the value: an escape sequence never
        // reaches a message.
        if let Some(character) = value.chars().find(|character| character.is_control()) {
            return Err(InvalidConfigFilePath::InvalidCharacter(character));
        }

        let Some(relative) = value.strip_prefix('/') else {
            return Err(InvalidConfigFilePath::Relative(value));
        };

        // Components become shell words and file names, so only printable
        // ASCII survives: no DEL and no wider Unicode.
        if let Some(character) = value
            .chars()
            .find(|character| *character == '\u{7f}' || !character.is_ascii())
        {
            return Err(InvalidConfigFilePath::InvalidCharacter(character));
        }

        if !has_plain_components(relative)
            || relative.split('/').any(|part| part.len() > MAX_NAME_BYTES)
        {
            return Err(InvalidConfigFilePath::InvalidComponent(value));
        }

        if !CONFIG_ROOTS.iter().any(|root| value.starts_with(root)) {
            return Err(InvalidConfigFilePath::OutsideRoots(value));
        }

        if WITHHELD_CONFIG_FILES
            .iter()
            .any(|prefix| value.starts_with(prefix))
            || WITHHELD_CONFIG_SUFFIXES
                .iter()
                .any(|suffix| value.ends_with(suffix))
        {
            return Err(InvalidConfigFilePath::Withheld(value));
        }

        Ok(Self(value))
    }

    /// A file directly in this folder, as the page lists them.
    ///
    /// # Errors
    ///
    /// [`InvalidConfigFilePath`] when `name` is no single plain component
    /// or the result breaks a rule of the type.
    pub fn join(&self, name: &str) -> Result<Self, InvalidConfigFilePath> {
        if name.contains('/') {
            return Err(InvalidConfigFilePath::InvalidComponent(name.to_owned()));
        }

        Self::new(format!("{}/{name}", self.0))
    }

    /// The path below `/`, for a walk from a root folder's descriptor.
    pub fn relative(&self) -> &str {
        self.0.strip_prefix('/').unwrap_or(&self.0)
    }

    /// The folder and the file name, both below `/`. Every path has both,
    /// because every root has a folder.
    pub fn split(&self) -> (&str, &str) {
        self.relative()
            .rsplit_once('/')
            .unwrap_or_else(|| ("", self.relative()))
    }

    /// Where the previous version is kept, below `/`.
    pub fn backup(&self) -> String {
        format!("{CONFIG_BACKUPS}/{}", self.relative())
    }
}
