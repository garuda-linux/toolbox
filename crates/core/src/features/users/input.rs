//! Text from the page's dialogs turned into value objects, with the code
//! the page shows a message for when a text breaks a rule.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_plan::{
    FullName, GroupName, HomeDirectory, InvalidFullName, InvalidHomeDirectory, InvalidName, Name,
    NewHome, NewUser, Reason, Rule, ShellPath, UserId,
};

use super::Refusal;
use super::home::{Placement, place};
use super::now::new_user;
use crate::Snapshot;

/// The code for a text that breaks a rule no field message names.
pub const INVALID: &str = "invalid";
/// The code for a text that is no user id a new account may ask for.
pub const UID_INVALID: &str = "uid-invalid";

/// The add-user dialog's fields, one typed argument per field.
///
/// An empty shell takes useradd's default, an empty `home` its default
/// path, an empty `uid` (a text, so no digit is lost on the way) the next
/// free id.
pub struct NewAccount<'fields> {
    pub name: &'fields str,
    pub full_name: &'fields str,
    pub shell: &'fields str,
    /// `None` when the page handed over no list of groups.
    pub groups: Option<BTreeSet<GroupName>>,
    pub home: &'fields str,
    pub no_home: bool,
    pub uid: &'fields str,
    pub system: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct AddForm<'fields> {
    pub name: &'fields str,
    pub name_ok: bool,
    pub first: &'fields str,
    pub second: &'fields str,
    pub password_ok: bool,
    pub rest_ok: bool,
}

/// What keeps "Add user now" from being clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddBlocker {
    NoName,
    BadName,
    NoPassword,
    PasswordsDiffer,
    /// A field says what is wrong below it.
    Fields,
}

impl AddBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoName => "no-name",
            Self::BadName => "bad-name",
            Self::NoPassword => "no-password",
            Self::PasswordsDiffer => "passwords-differ",
            Self::Fields => "fields",
        }
    }
}

/// Why a typed account name breaks the rule of shadow-utils.
pub const fn name_code(error: &InvalidName) -> &'static str {
    match error.reason() {
        Reason::InvalidComponent | Reason::WrongFileType => INVALID,
        reason @ (Reason::Empty
        | Reason::TooLong { max: _ }
        | Reason::InvalidStart(_)
        | Reason::InvalidCharacter(_)
        | Reason::UnknownUnitType
        | Reason::UnknownDevice) => crate::features::reason_code(reason),
    }
}

/// A user or group name, or the code of the rule it breaks.
///
/// # Errors
///
/// The code of the rule `text` breaks.
pub fn name<R>(text: &str) -> Result<Name<R>, &'static str>
where
    R: Rule,
{
    Name::new(text).map_err(|error| name_code(&error))
}

/// The user id typed into the add-user dialog. Blank lets useradd pick.
///
/// # Errors
///
/// [`UID_INVALID`] for what is no number between 1 and
/// [`toolbox_plan::LAST_USER_ID`].
pub fn user_id(text: &str) -> Result<Option<UserId>, &'static str> {
    match text.trim() {
        "" => Ok(None),
        digits => digits
            .parse::<u32>()
            .ok()
            .and_then(|value| UserId::new(value).ok())
            .map(Some)
            .ok_or(UID_INVALID),
    }
}

/// A typed home folder, or the code of the rule it breaks.
///
/// # Errors
///
/// The code of the rule `text` breaks.
pub fn home(text: &str) -> Result<HomeDirectory, &'static str> {
    HomeDirectory::new(text).map_err(InvalidHomeDirectory::as_str)
}

/// The add-user dialog's fields as a user to create.
///
/// # Errors
///
/// The code of the first field that breaks a rule, or of the guard that
/// refuses the user.
pub fn new_account(form: &NewAccount<'_>, snapshot: &Snapshot) -> Result<NewUser, &'static str> {
    let user = name(form.name)?;
    let full_name = FullName::new(form.full_name).map_err(InvalidFullName::as_str)?;
    let shell = match form.shell {
        "" => None,
        path => Some(ShellPath::new(path).map_err(|_invalid| Refusal::ShellNotListed.as_str())?),
    };

    let groups = form.groups.clone().ok_or(INVALID)?;
    let placement = Placement {
        home: match (form.no_home, form.home) {
            (true, _) => NewHome::Absent,
            (false, "") => NewHome::Standard,
            (false, path) => NewHome::At(home(path)?),
        },
        uid: user_id(form.uid)?,
        system: form.system,
    };

    let user = new_user(user, full_name, shell, groups, snapshot).map_err(Refusal::as_str)?;

    place(user, placement, snapshot).map_err(Refusal::as_str)
}

/// The first thing that keeps "Add user now" from being clicked, in the
/// order of the dialog's fields.
pub fn add_blocker(form: &AddForm<'_>) -> Option<AddBlocker> {
    if form.name.is_empty() {
        Some(AddBlocker::NoName)
    } else if !form.name_ok {
        Some(AddBlocker::BadName)
    } else if form.first.is_empty() {
        Some(AddBlocker::NoPassword)
    } else if form.first != form.second {
        Some(AddBlocker::PasswordsDiffer)
    } else if !form.password_ok || !form.rest_ok {
        Some(AddBlocker::Fields)
    } else {
        None
    }
}
