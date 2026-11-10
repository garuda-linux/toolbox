//! A value from the Boot options page, checked against its rule and
//! queued. The window and the CLI hand their raw values in here, so a
//! broken one names the same rule in each.

#[cfg(test)]
mod tests;

use toolbox_plan::{
    DefaultEntry, GrubPicture, GrubSetting, GrubThemeFile, GrubTimeout, InvalidName,
    KernelParameter, MenuEntryId, PlymouthTheme, Reason,
};

use super::{
    Flag, Refusal, add_parameter, remove_parameter, set, set_flag, set_plymouth_theme, set_remember,
};
use crate::features::reason_code;
use crate::{ChangeSet, Snapshot, Toggle};

/// One change the page asks for, as the user typed or picked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    Default(String),
    /// Seconds, 0 to 300.
    Timeout(i64),
    /// `fallback` is the entry that stays the default when remembering
    /// is switched off again.
    Remember {
        on: bool,
        fallback: String,
    },
    Submenus(bool),
    OsProber(bool),
    /// `confirmed` once the user said yes to a risky one.
    AddParameter {
        text: String,
        confirmed: bool,
    },
    RemoveParameter(String),
    Flag(Flag, bool),
    /// A picture's full path, `""` for none.
    Background(String),
    /// A `theme.txt`'s full path, `""` for none.
    Theme(String),
    PlymouthTheme(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditProblem {
    Refused(Refusal),
    Invalid(InvalidName),
    OutOfRange,
    /// One of `RISKY_PARAMETERS` that the user has not confirmed yet.
    NeedsConfirm,
}

impl Edit {
    /// The value as the user gave it, for a refusal that names it. A
    /// switch has none.
    pub fn value(&self) -> String {
        match self {
            Self::Default(text)
            | Self::AddParameter { text, confirmed: _ }
            | Self::RemoveParameter(text)
            | Self::Background(text)
            | Self::Theme(text)
            | Self::PlymouthTheme(text)
            | Self::Remember {
                on: _,
                fallback: text,
            } => text.clone(),
            Self::Timeout(seconds) => seconds.to_string(),
            Self::Submenus(_) | Self::OsProber(_) | Self::Flag(_, _) => String::new(),
        }
    }
}

impl EditProblem {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Refused(refusal) => refusal.as_str(),
            Self::Invalid(error) => rule(error),
            Self::OutOfRange => "invalid",
            Self::NeedsConfirm => "needs-confirm",
        }
    }
}

impl From<InvalidName> for EditProblem {
    fn from(error: InvalidName) -> Self {
        Self::Invalid(error)
    }
}

/// # Errors
///
/// When the value breaks its rule, or the system or the queue refuses it.
pub fn edit(change: Edit, changes: &mut ChangeSet, snapshot: &Snapshot) -> Result<(), EditProblem> {
    let queued = match change {
        Edit::Default(id) => set(
            GrubSetting::Default(DefaultEntry::Entry(MenuEntryId::new(id)?)),
            changes,
            snapshot,
        ),
        Edit::Timeout(seconds) => {
            let timeout = GrubTimeout::new(seconds).map_err(|_out| EditProblem::OutOfRange)?;
            set(GrubSetting::Timeout(timeout), changes, snapshot)
        }
        Edit::Remember { on, fallback } => set_remember(
            Toggle::from_on(on),
            MenuEntryId::new(fallback)?,
            changes,
            snapshot,
        ),
        Edit::Submenus(on) => set(GrubSetting::Submenus(on), changes, snapshot),
        Edit::OsProber(on) => set(GrubSetting::OsProber(on), changes, snapshot),
        Edit::AddParameter { text, confirmed } => {
            let parameter = KernelParameter::new(text)?;

            if parameter.is_risky() && !confirmed {
                return Err(EditProblem::NeedsConfirm);
            }

            add_parameter(parameter, changes, snapshot)
        }
        Edit::RemoveParameter(text) => remove_parameter(&text, changes, snapshot),
        Edit::Flag(flag, on) => set_flag(flag, Toggle::from_on(on), changes, snapshot),
        Edit::Background(path) => {
            let picture = optional(path, GrubPicture::new)?;
            set(GrubSetting::Background(picture), changes, snapshot)
        }
        Edit::Theme(path) => set(
            GrubSetting::Theme(optional(path, GrubThemeFile::new)?),
            changes,
            snapshot,
        ),
        Edit::PlymouthTheme(name) => {
            set_plymouth_theme(PlymouthTheme::new(name)?, changes, snapshot)
        }
    };

    queued.map_err(EditProblem::Refused)
}

/// `None` for an empty `path`, which takes the file away.
fn optional<T>(
    path: String,
    parse: impl FnOnce(String) -> Result<T, InvalidName>,
) -> Result<Option<T>, InvalidName> {
    if path.is_empty() {
        Ok(None)
    } else {
        parse(path).map(Some)
    }
}

const fn rule(error: &InvalidName) -> &'static str {
    match error.reason() {
        // The page words a character the same wherever it stands.
        Reason::InvalidStart(character) => reason_code(Reason::InvalidCharacter(character)),
        reason @ (Reason::Empty
        | Reason::TooLong { max: _ }
        | Reason::InvalidCharacter(_)
        | Reason::InvalidComponent
        | Reason::WrongFileType
        | Reason::UnknownUnitType
        | Reason::UnknownDevice) => reason_code(reason),
    }
}
