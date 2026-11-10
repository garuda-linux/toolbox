//! Validated names of system objects (packages, units, users, …).
//!
//! Every name ends up as an argument of a program that may run as root, so
//! the rules reject anything a program could read as an option or a shell
//! could split: a leading `-`, whitespace, separators and quotes.

mod boot;
mod disk;
mod known;
mod loader;
mod rules;
#[cfg(test)]
mod tests;

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub use boot::{GrubPicture, GrubThemeFile, KernelParameter, MenuEntryId, PlymouthTheme};
pub use disk::{BootloaderId, DiskDevice, PartitionDevice, VolumeDevice};
pub use known::KnownPackage;
pub use loader::{LoaderDefault, LoaderEntryId};
pub use rules::{
    ADMIN_GROUPS, Charset, DkmsModuleName, DkmsVersion, GroupName, HostName, KernelModule,
    KernelRelease, LocaleName, PackageName, UnitName, UserName,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::shown::Shown;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameKind {
    Package,
    Unit,
    User,
    Group,
    Locale,
    KernelModule,
    Charset,
    DkmsModule,
    DkmsVersion,
    KernelRelease,
    Host,
    KernelParameter,
    PlymouthTheme,
    MenuEntry,
    GrubPicture,
    GrubTheme,
    Disk,
    Partition,
    Volume,
    Mapper,
    BootloaderId,
    LoaderEntry,
    LoaderDefault,
}

impl fmt::Display for NameKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Package => "package name",
            Self::Unit => "unit name",
            Self::User => "user name",
            Self::Group => "group name",
            Self::Locale => "locale name",
            Self::KernelModule => "kernel module name",
            Self::Charset => "character set name",
            Self::DkmsModule => "DKMS module name",
            Self::DkmsVersion => "DKMS module version",
            Self::KernelRelease => "kernel release",
            Self::Host => "host name or address",
            Self::KernelParameter => "kernel parameter",
            Self::PlymouthTheme => "Plymouth theme name",
            Self::MenuEntry => "boot menu entry",
            Self::GrubPicture => "GRUB background picture",
            Self::GrubTheme => "GRUB theme file",
            Self::Disk => "disk",
            Self::Partition => "partition",
            Self::Volume => "partition or mapped device",
            Self::Mapper => "device-mapper name",
            Self::BootloaderId => "boot loader name",
            Self::LoaderEntry => "systemd-boot entry",
            Self::LoaderDefault => "systemd-boot default entry",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Empty,
    TooLong { max: usize },
    InvalidStart(char),
    InvalidCharacter(char),
    UnknownUnitType,
    InvalidComponent,
    WrongFileType,
    UnknownDevice,
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("it is empty"),
            Self::TooLong { max } => write!(f, "it is longer than {max} characters"),
            Self::InvalidStart(character) => {
                write!(f, "it must not start with `{character}`")
            }
            Self::InvalidCharacter(character) => {
                write!(f, "it contains `{}`", character.escape_debug())
            }
            Self::UnknownUnitType => f.write_str("it needs a unit type suffix such as `.service`"),
            Self::InvalidComponent => {
                f.write_str("it is no absolute path without `.` and `..` parts")
            }
            Self::WrongFileType => f.write_str("its file type does not fit"),
            Self::UnknownDevice => {
                f.write_str("it is no disk, partition or mapped device as the kernel names them")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid {kind} `{}`: {reason}", Shown::new(.value))]
pub struct InvalidName {
    kind: NameKind,
    value: String,
    reason: Reason,
}

impl InvalidName {
    pub fn new(kind: NameKind, value: impl Into<String>, reason: Reason) -> Self {
        Self {
            kind,
            value: value.into(),
            reason,
        }
    }

    pub const fn reason(&self) -> Reason {
        self.reason
    }
}

pub trait Rule {
    const KIND: NameKind;

    /// # Errors
    ///
    /// The [`Reason`] the value breaks the rule.
    fn check(value: &str) -> Result<(), Reason>;

    /// The one spelling of a value that passed [`Rule::check`], for kinds
    /// where the tools read two spellings as one name.
    fn normalize(value: String) -> String {
        value
    }
}

/// A string that passed the rule `R`. The only way to get one is
/// [`Name::new`] (or deserializing, which calls it).
pub struct Name<R> {
    value: String,
    rule: PhantomData<fn() -> R>,
}

impl<R> Name<R>
where
    R: Rule,
{
    /// # Errors
    ///
    /// [`InvalidName`] when `value` breaks the rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use toolbox_plan::PackageName;
    ///
    /// assert_eq!(PackageName::new("linux-zen").unwrap().as_str(), "linux-zen");
    /// assert!(PackageName::new("vim;rm").is_err());
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidName> {
        let value = value.into();
        match R::check(&value) {
            Ok(()) => Ok(Self {
                value: R::normalize(value),
                rule: PhantomData,
            }),
            Err(reason) => Err(InvalidName::new(R::KIND, value, reason)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl<R> Clone for Name<R> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            rule: PhantomData,
        }
    }
}

impl<R> PartialEq for Name<R> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<R> Eq for Name<R> {}

impl<R> PartialOrd for Name<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R> Ord for Name<R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<R> Hash for Name<R> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.value.hash(state);
    }
}

impl<R> fmt::Debug for Name<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.value, f)
    }
}

impl<R> fmt::Display for Name<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl<R> AsRef<str> for Name<R> {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl<R> std::str::FromStr for Name<R>
where
    R: Rule,
{
    type Err = InvalidName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<R> TryFrom<String> for Name<R>
where
    R: Rule,
{
    type Error = InvalidName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<R> Serialize for Name<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

impl<'de, R> Deserialize<'de> for Name<R>
where
    R: Rule,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}
