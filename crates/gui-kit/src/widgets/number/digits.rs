//! The text of a number field while it is typed. Only digits go in, so a
//! refused letter never moves the caret.

use std::any::TypeId;
use std::ops::Range;

use egui::TextBuffer;
use egui::epaint::text::CharIndex;

#[derive(Clone, Debug, Default)]
pub(super) struct Digits(pub(super) String);

impl TextBuffer for Digits {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        &self.0
    }

    /// Inserts the digits of `text` and drops the rest.
    fn insert_text(&mut self, text: &str, char_index: CharIndex) -> usize {
        let digits: String = text.chars().filter(char::is_ascii_digit).collect();
        let byte = self.byte_index_from_char_index(char_index);
        self.0.insert_str(byte.0, &digits);
        digits.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<CharIndex>) {
        let start = self.byte_index_from_char_index(char_range.start);
        let end = self.byte_index_from_char_index(char_range.end);
        self.0.drain(start.0..end.0);
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}
