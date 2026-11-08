#[cfg(test)]
mod tests;

use std::fmt;

const SHOWN_CHARACTERS: usize = 64;

/// The start of a refused value with its control characters spelled out.
/// The value is the caller's: megabytes long, or escape sequences for the
/// terminal and the log that show the message.
pub struct Shown<'value> {
    value: &'value str,
    limit: usize,
}

impl<'value> Shown<'value> {
    pub const fn new(value: &'value str) -> Self {
        Self::up_to(value, SHOWN_CHARACTERS)
    }

    /// For a text that is a message of its own, such as a decoder's error,
    /// which needs more room than a value.
    pub const fn up_to(value: &'value str, limit: usize) -> Self {
        Self { value, limit }
    }
}

impl fmt::Display for Shown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut characters = self.value.chars();

        for character in characters.by_ref().take(self.limit) {
            write!(f, "{}", character.escape_debug())?;
        }

        if characters.next().is_some() {
            f.write_str("…")?;
        }

        Ok(())
    }
}
