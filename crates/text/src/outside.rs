use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Text from outside that holds no hidden character. Its own type, so
/// text that was never [`plain`](crate::plain) cannot be shown.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OutsideText(String);

impl OutsideText {
    /// `text` with its hidden characters dropped, but its line breaks and
    /// tabs kept. For file text, diffs and multi-line details.
    pub fn keep_lines(text: &str) -> Self {
        Self(
            text.chars()
                .filter(|character| {
                    matches!(character, '\n' | '\t') || !crate::is_hidden(*character)
                })
                .collect(),
        )
    }

    pub(crate) const fn from_visible(text: String) -> Self {
        Self(text)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Deref for OutsideText {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for OutsideText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OutsideText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<str> for OutsideText {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for OutsideText {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Serialize for OutsideText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

/// Read text goes through [`crate::plain`] too, so a stored or sent text
/// cannot bring hidden characters back past the type. It comes back as one
/// line, so a text of [`OutsideText::keep_lines`] keeps no line break.
impl<'de> Deserialize<'de> for OutsideText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;

        Ok(crate::plain(&text))
    }
}
