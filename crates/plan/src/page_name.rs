//! The name of a page or of a page action, such as `system-settings` or
//! `inxi`. The command line's `--page` and `--action` and a later start's
//! wish over the session bus share this one rule.

#[cfg(test)]
mod tests;

/// Longer names name no page of the app. The bound also keeps a caller on
/// the bus from parking large strings in the app.
pub const MAX_PAGE_NAME_BYTES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPageName {
    #[error("the name is empty")]
    Empty,
    #[error("the name is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the name contains `{}`, only small letters and hyphens", .0.escape_debug())]
    InvalidCharacter(char),
}

/// A short word of small letters and hyphens.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageName(String);

validated_string!(PageName, InvalidPageName);

impl PageName {
    /// # Errors
    ///
    /// [`InvalidPageName`] for an empty or overlong text, or one with
    /// anything but small ASCII letters and hyphens.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidPageName> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidPageName::Empty);
        }

        if value.len() > MAX_PAGE_NAME_BYTES {
            return Err(InvalidPageName::TooLong {
                max: MAX_PAGE_NAME_BYTES,
            });
        }

        if let Some(character) = value
            .chars()
            .find(|character| !(character.is_ascii_lowercase() || *character == '-'))
        {
            return Err(InvalidPageName::InvalidCharacter(character));
        }

        Ok(Self(value))
    }
}
