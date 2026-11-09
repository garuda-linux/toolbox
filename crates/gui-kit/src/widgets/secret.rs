//! The text of a password field. It is overwritten when dropped and never
//! grows past the room it starts with, so no reallocation leaves a copy of
//! the password behind in freed memory.

#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::fmt;
use std::ops::Range;

use egui::TextBuffer;
use egui::epaint::text::CharIndex;
use zeroize::Zeroizing;

pub struct SecretText(Zeroizing<String>);

impl SecretText {
    /// Room for `bytes`. Typing past it is ignored.
    pub fn with_room(bytes: usize) -> Self {
        Self(Zeroizing::new(String::with_capacity(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretText(<redacted>)")
    }
}

impl TextBuffer for SecretText {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        &self.0
    }

    /// Inserts the whole characters of `text` that still fit the room.
    fn insert_text(&mut self, text: &str, char_index: CharIndex) -> usize {
        let room = self.0.capacity() - self.0.len();
        let fitting = text
            .char_indices()
            .map(|(start, character)| start + character.len_utf8())
            .take_while(|end| *end <= room)
            .last()
            .unwrap_or(0);

        let fits = text.get(..fitting).unwrap_or_default();
        let byte = self.byte_index_from_char_index(char_index);

        self.0.insert_str(byte.0, fits);
        fits.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<CharIndex>) {
        let start = self.byte_index_from_char_index(char_range.start);
        let end = self.byte_index_from_char_index(char_range.end);

        self.0.drain(start.0..end.0);
    }

    /// Hands out nothing: a copy would be a plain `String` no one clears.
    fn take(&mut self) -> String {
        self.clear();
        String::new()
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}
