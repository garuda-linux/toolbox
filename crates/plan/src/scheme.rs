#[cfg(test)]
mod tests;

use std::fmt;

/// Longer ids name no scheme of the catalog. The bound also keeps a
/// settings file from holding a page of text in one key.
pub const MAX_SCHEME_ID_BYTES: usize = 48;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidSchemeId {
    #[error("the scheme id is empty")]
    Empty,
    #[error("the scheme id is longer than {max} bytes")]
    TooLong { max: usize },
    #[error("the scheme id must be lowercase words joined by dashes, such as catppuccin-mocha")]
    NotWords,
}

/// The id of a colour scheme, lowercase words of ASCII letters and digits
/// joined by single dashes, such as `catppuccin-mocha`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemeId(String);

validated_string!(SchemeId, InvalidSchemeId);

impl SchemeId {
    /// # Errors
    ///
    /// [`InvalidSchemeId`] for an empty or overlong text, or one that is
    /// not lowercase words joined by single dashes.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidSchemeId> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidSchemeId::Empty);
        }

        if value.len() > MAX_SCHEME_ID_BYTES {
            return Err(InvalidSchemeId::TooLong {
                max: MAX_SCHEME_ID_BYTES,
            });
        }

        let words = value.split('-').all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit())
        });

        if !words {
            return Err(InvalidSchemeId::NotWords);
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is no colour of the form #rrggbb")]
pub struct InvalidRgb(pub String);

/// An opaque sRGB colour as a scheme lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb([u8; 3]);

impl Rgb {
    /// # Errors
    ///
    /// [`InvalidRgb`] unless `text` is `#` and six hex digits.
    pub fn parse(text: &str) -> Result<Self, InvalidRgb> {
        let invalid = || InvalidRgb(text.to_owned());
        let digits = text.strip_prefix('#').ok_or_else(invalid)?;

        if digits.len() != 6 || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
            return Err(invalid());
        }

        let channel = |at: usize| {
            digits
                .get(at..at + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .ok_or_else(invalid)
        };

        Ok(Self([channel(0)?, channel(2)?, channel(4)?]))
    }

    pub const fn bytes(self) -> [u8; 3] {
        self.0
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [red, green, blue] = self.0;
        write!(f, "#{red:02x}{green:02x}{blue:02x}")
    }
}
