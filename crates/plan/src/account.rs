mod home;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

pub use home::{
    HomeChange, HomeDirectory, InvalidHomeDirectory, InvalidUserId, LAST_USER_ID, NewHome, UserId,
};
use serde::{Deserialize, Serialize};

use crate::{GroupName, ShellPath, UserName};

/// Far above any real name, far below what `/etc/passwd` lines may hold.
pub const MAX_FULL_NAME_BYTES: usize = 128;

/// `/etc/shadow`, relative to `/`. Only the helper reads it.
pub const SHADOW: &str = "etc/shadow";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidFullName {
    #[error("the full name is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the full name contains `{}`", .0.escape_debug())]
    InvalidCharacter(char),
    #[error("the full name must not start with `-`")]
    LeadingDash,
    #[error("the full name starts or ends with a blank")]
    OuterBlank,
}

impl InvalidFullName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TooLong { max: _ } => "too-long",
            Self::InvalidCharacter(_) => "invalid-character",
            Self::LeadingDash => "leading-dash",
            Self::OuterBlank => "outer-blank",
        }
    }
}

/// The first part of an account's GECOS field. May be empty.
///
/// It ends up in `/etc/passwd` and in the argv of `useradd` and `chfn`, so it
/// holds no `:` (the file's separator), no `,` or `=` (GECOS separators), no
/// `"` (chfn refuses it) and no control character.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullName(String);

validated_string!(FullName, InvalidFullName);

impl FullName {
    /// # Errors
    ///
    /// [`InvalidFullName`] for an overlong text, one with a separator or
    /// control character, a leading `-` or blanks around it.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidFullName> {
        let value = value.into();

        if value.len() > MAX_FULL_NAME_BYTES {
            return Err(InvalidFullName::TooLong {
                max: MAX_FULL_NAME_BYTES,
            });
        }

        if let Some(character) = value
            .chars()
            .find(|character| character.is_control() || ":,=\"".contains(*character))
        {
            return Err(InvalidFullName::InvalidCharacter(character));
        }

        if value.starts_with('-') {
            return Err(InvalidFullName::LeadingDash);
        }

        if value.trim() != value {
            return Err(InvalidFullName::OuterBlank);
        }

        Ok(Self(value))
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidNewUser {
    #[error("the user id {0} is not in the range of system accounts")]
    IdOfAPerson(UserId),
    #[error("the user id {0} is in the range of system accounts")]
    IdOfTheSystem(UserId),
}

/// Its parts fit each other: it only comes from [`NewUser::new`], also when decoded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NewUserParts", into = "NewUserParts")]
pub struct NewUser(NewUserParts);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewUserParts {
    pub name: UserName,
    pub full_name: FullName,
    /// `None` takes useradd's default from `/etc/default/useradd`.
    pub shell: Option<ShellPath>,
    pub groups: BTreeSet<GroupName>,
    pub home: NewHome,
    pub uid: Option<UserId>,
    /// An account of a service: useradd takes its id from the system
    /// range and makes no entries for sub ids.
    pub system: bool,
}

impl NewUser {
    /// An asked for user id must lie in the range of the kind of account,
    /// so no person gets the id of a service and no service one of a
    /// person.
    ///
    /// # Errors
    ///
    /// [`InvalidNewUser`] when it does not.
    pub fn new(parts: NewUserParts) -> Result<Self, InvalidNewUser> {
        match parts.uid {
            Some(uid) if parts.system && !uid.is_system() => Err(InvalidNewUser::IdOfAPerson(uid)),
            Some(uid) if !parts.system && uid.is_system() => {
                Err(InvalidNewUser::IdOfTheSystem(uid))
            }
            Some(_) | None => Ok(Self(parts)),
        }
    }

    pub const fn parts(&self) -> &NewUserParts {
        &self.0
    }

    pub fn into_parts(self) -> NewUserParts {
        self.0
    }
}

impl TryFrom<NewUserParts> for NewUser {
    type Error = InvalidNewUser;

    fn try_from(parts: NewUserParts) -> Result<Self, InvalidNewUser> {
        Self::new(parts)
    }
}

impl From<NewUser> for NewUserParts {
    fn from(user: NewUser) -> Self {
        user.0
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum HomeFolder {
    #[default]
    Keep,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountLock {
    pub user: UserName,
    pub locked: bool,
}

/// Names and flags only, never a hash.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowLocks {
    pub locks: Vec<AccountLock>,
}

impl ShadowLocks {
    /// The lock flag of every entry of an `/etc/shadow` text whose name is
    /// a valid [`UserName`]. Locked means what `passwd -S` calls `L` after
    /// `usermod -L`: the password field starts with `!`. Nothing else of
    /// the field leaves this function.
    pub fn from_shadow(shadow: &str) -> Self {
        let locks = shadow
            .lines()
            .filter_map(|line| {
                let mut fields = line.split(':');
                let user = UserName::new(fields.next()?).ok()?;
                let locked = fields.next()?.starts_with('!');

                Some(AccountLock { user, locked })
            })
            .collect();

        Self { locks }
    }
}
