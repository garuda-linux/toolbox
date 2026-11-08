//! The name rules of systemd-boot. `loader.conf` takes one word after a
//! setting's name, so none of these holds a blank or a line break.

#[cfg(test)]
mod tests;

use super::rules::{check_characters, check_shape};
use super::{Name, NameKind, Reason, Rule};

const ENTRY_MARKS: &str = "._+-@~";

const GLOB_MARKS: &str = "*?[]";

const MAX_ENTRY_CHARS: usize = 255;

pub type LoaderEntryId = Name<LoaderEntryRule>;
pub type LoaderDefault = Name<LoaderDefaultRule>;

pub struct LoaderEntryRule;
pub struct LoaderDefaultRule;

impl Rule for LoaderEntryRule {
    const KIND: NameKind = NameKind::LoaderEntry;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_ENTRY_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || ENTRY_MARKS.contains(character)
        })
    }
}

impl Rule for LoaderDefaultRule {
    const KIND: NameKind = NameKind::LoaderDefault;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_ENTRY_CHARS, |first| {
            first.is_ascii_alphanumeric() || "@*?[".contains(first)
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric()
                || ENTRY_MARKS.contains(character)
                || GLOB_MARKS.contains(character)
        })
    }
}

/// Every character an entry id may hold is one a default may hold too.
impl From<&LoaderEntryId> for LoaderDefault {
    fn from(id: &LoaderEntryId) -> Self {
        Self {
            value: id.as_str().to_owned(),
            rule: std::marker::PhantomData,
        }
    }
}

impl Name<LoaderDefaultRule> {
    pub fn saved() -> Self {
        Self {
            value: "@saved".to_owned(),
            rule: std::marker::PhantomData,
        }
    }
}
