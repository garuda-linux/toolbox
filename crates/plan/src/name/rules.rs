//! The concrete name rules. Limits follow the tools that consume the names:
//! makepkg for packages, systemd for units, shadow-utils' default
//! `NAME_REGEX` for accounts, glibc for locales and charsets, the kernel's
//! `MODULE_NAME_LEN` for modules, what dkms and `uname -r` print for DKMS
//! modules and kernel releases, and DNS for hosts.

use super::{Name, NameKind, Reason, Rule};

/// Groups whose members may become root with sudo.
///
/// `wheel` on Arch and Garuda, `sudo` where an admin set the system up that
/// way. A group named only in a sudoers file is not seen, the app cannot
/// read that file.
// toolbox-lint: allow(sudo-once) A group name, no program.
pub const ADMIN_GROUPS: [&str; 2] = ["wheel", "sudo"];

const UNIT_TYPES: [&str; 11] = [
    ".service",
    ".socket",
    ".timer",
    ".target",
    ".mount",
    ".automount",
    ".path",
    ".slice",
    ".scope",
    ".device",
    ".swap",
];

const MAX_PACKAGE_CHARS: usize = 255;
const MAX_UNIT_CHARS: usize = 255;
const MAX_ACCOUNT_CHARS: usize = 32;
const MAX_LOCALE_CHARS: usize = 64;
const MAX_CHARSET_CHARS: usize = 32;
/// The kernel's `MODULE_NAME_LEN` is 56 with the final NUL.
const MAX_KERNEL_MODULE_CHARS: usize = 55;
const MAX_DKMS_CHARS: usize = 64;
const MAX_KERNEL_RELEASE_CHARS: usize = 64;
const MAX_HOST_CHARS: usize = 253;

pub type PackageName = Name<PackageRule>;
pub type UnitName = Name<UnitRule>;
pub type UserName = Name<UserRule>;
pub type GroupName = Name<GroupRule>;
pub type LocaleName = Name<LocaleRule>;
pub type KernelModule = Name<KernelModuleRule>;
pub type Charset = Name<CharsetRule>;
pub type DkmsModuleName = Name<DkmsModuleRule>;
pub type DkmsVersion = Name<DkmsVersionRule>;
pub type KernelRelease = Name<KernelReleaseRule>;
pub type HostName = Name<HostRule>;

pub struct PackageRule;
pub struct UnitRule;
/// Users and groups share shadow-utils' rule but stay distinct types, so a
/// user can never be passed where a group belongs.
pub struct UserRule;
pub struct GroupRule;
pub struct LocaleRule;
pub struct KernelModuleRule;
/// The second word of a `locale.gen` line, a file name below
/// `/usr/share/i18n/charmaps`.
pub struct CharsetRule;
pub struct DkmsModuleRule;
pub struct DkmsVersionRule;
/// What `uname -r` prints, also the folder name below `/usr/lib/modules`.
pub struct KernelReleaseRule;
/// A DNS name or an IPv4 or IPv6 address, as ping and traceroute take it.
pub struct HostRule;

impl GroupName {
    pub fn is_admin(&self) -> bool {
        ADMIN_GROUPS.contains(&self.value.as_str())
    }
}

impl Rule for PackageRule {
    const KIND: NameKind = NameKind::Package;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_PACKAGE_CHARS, |first| {
            first.is_ascii_alphanumeric() || "@_+".contains(first)
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || "@._+-".contains(character)
        })
    }
}

impl Rule for UnitRule {
    const KIND: NameKind = NameKind::Unit;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_UNIT_CHARS, |first| {
            first.is_ascii_alphanumeric() || "_@".contains(first)
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || ":_.@\\-".contains(character)
        })?;
        if UNIT_TYPES
            .iter()
            .any(|suffix| value.len() > suffix.len() && value.ends_with(suffix))
        {
            Ok(())
        } else {
            Err(Reason::UnknownUnitType)
        }
    }
}

impl Rule for UserRule {
    const KIND: NameKind = NameKind::User;

    fn check(value: &str) -> Result<(), Reason> {
        check_account(value)
    }
}

impl Rule for GroupRule {
    const KIND: NameKind = NameKind::Group;

    fn check(value: &str) -> Result<(), Reason> {
        check_account(value)
    }
}

impl Rule for LocaleRule {
    const KIND: NameKind = NameKind::Locale;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_LOCALE_CHARS, |first| first.is_ascii_alphabetic())?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || "_.@-".contains(character)
        })
    }
}

impl Rule for KernelModuleRule {
    const KIND: NameKind = NameKind::KernelModule;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_KERNEL_MODULE_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || "_-".contains(character)
        })
    }

    /// The kernel and modprobe read `-` and `_` in a module name as the
    /// same character. With `_` only, one module has one spelling.
    fn normalize(value: String) -> String {
        value.replace('-', "_")
    }
}

impl Rule for CharsetRule {
    const KIND: NameKind = NameKind::Charset;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_CHARSET_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_word(value)
    }
}

impl Rule for DkmsModuleRule {
    const KIND: NameKind = NameKind::DkmsModule;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_DKMS_CHARS, |first| first.is_ascii_alphanumeric())?;
        check_word(value)
    }
}

impl Rule for DkmsVersionRule {
    const KIND: NameKind = NameKind::DkmsVersion;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_DKMS_CHARS, |first| first.is_ascii_alphanumeric())?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || "._+~-".contains(character)
        })
    }
}

impl Rule for KernelReleaseRule {
    const KIND: NameKind = NameKind::KernelRelease;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_KERNEL_RELEASE_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_word(value)
    }
}

/// `:` is for IPv6 addresses, which may start with it (`::1`). No
/// `%zone`: a link-local target is no use for a connection test.
impl Rule for HostRule {
    const KIND: NameKind = NameKind::Host;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_HOST_CHARS, |first| {
            first.is_ascii_alphanumeric() || first == ':'
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || ".-:".contains(character)
        })
    }
}

/// Letters, digits and `._+-`: one path component that is neither `.` nor
/// `..`, since the first character is a letter or digit.
pub fn check_word(value: &str) -> Result<(), Reason> {
    check_characters(value, |character| {
        character.is_ascii_alphanumeric() || "._+-".contains(character)
    })
}

fn check_account(value: &str) -> Result<(), Reason> {
    check_shape(value, MAX_ACCOUNT_CHARS, |first| {
        first.is_ascii_lowercase() || first == '_'
    })?;
    check_characters(value, |character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || "_-".contains(character)
    })
}

/// Not empty, at most `max` characters, and a first character that passes
/// `first_ok`.
pub fn check_shape(value: &str, max: usize, first_ok: impl Fn(char) -> bool) -> Result<(), Reason> {
    let Some(first) = value.chars().next() else {
        return Err(Reason::Empty);
    };

    if value.chars().count() > max {
        return Err(Reason::TooLong { max });
    }

    if first_ok(first) {
        Ok(())
    } else {
        Err(Reason::InvalidStart(first))
    }
}

pub fn check_characters(value: &str, allowed: impl Fn(char) -> bool) -> Result<(), Reason> {
    value
        .chars()
        .find(|character| !allowed(*character))
        .map_or(Ok(()), |character| Err(Reason::InvalidCharacter(character)))
}
