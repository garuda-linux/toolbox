//! Heading ids as mdBook makes them, so a link written for the book also
//! finds its heading in the app.

use std::collections::HashMap;

/// Hands out unique ids on one page: the second `steps` is `steps-1`.
#[derive(Debug, Default)]
pub(super) struct Anchors {
    seen: HashMap<String, usize>,
}

impl Anchors {
    pub(super) fn unique(&mut self, text: &str) -> String {
        let base = slug(text);
        let count = self.seen.entry(base.clone()).or_insert(0);
        let anchor = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };

        *count += 1;
        anchor
    }
}

/// The id of a heading with the words `text`: lower case, spaces turned
/// into `-`, everything but letters, digits, `-` and `_` left out.
pub fn slug(text: &str) -> String {
    text.trim()
        .chars()
        .filter_map(|char| {
            if char.is_whitespace() {
                Some('-')
            } else if char.is_alphanumeric() || char == '_' || char == '-' {
                Some(char)
            } else {
                None
            }
        })
        .flat_map(char::to_lowercase)
        .collect()
}
