//! A named argument of a Fluent message: a text stays a text, a count
//! is a number the message picks its plural form by.

use std::borrow::Cow;

use fluent_templates::fluent_bundle::FluentValue;

macro_rules! counts {
    ($($number:ty),*) => {
        $(impl From<$number> for Arg<'_> {
            fn from(count: $number) -> Self {
                Self::Count(i64::try_from(count).unwrap_or(i64::MAX))
            }
        }

        impl From<&$number> for Arg<'_> {
            fn from(count: &$number) -> Self {
                Self::from(*count)
            }
        })*
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg<'a> {
    Text(&'a str),
    Count(i64),
}

impl Arg<'_> {
    pub(super) fn value(self) -> FluentValue<'static> {
        match self {
            Self::Text(text) => FluentValue::from(text.to_owned()),
            Self::Count(count) => FluentValue::from(count),
        }
    }
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(text: &'a str) -> Self {
        Self::Text(text)
    }
}

impl<'a> From<&'a String> for Arg<'a> {
    fn from(text: &'a String) -> Self {
        Self::Text(text)
    }
}

impl<'a> From<&'a Cow<'_, str>> for Arg<'a> {
    fn from(text: &'a Cow<'_, str>) -> Self {
        Self::Text(text)
    }
}

counts!(u8, u16, u32, u64, usize, i32, i64);
