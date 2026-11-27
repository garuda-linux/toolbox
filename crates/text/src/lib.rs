//! Text from outside (a server's answer, a package description, a file a
//! user wrote). [`OutsideText`] is the only type that marks it, and only
//! [`plain`] and [`OutsideText::keep_lines`] make one, so a field typed
//! `OutsideText` cannot hold a raw string by accident.

#[cfg(test)]
mod tests;

mod outside;

pub use self::outside::OutsideText;

/// Whether `character` must not reach the display, as it would change how
/// the text around it reads.
pub fn is_hidden(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
                | '\u{200b}'..='\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{2060}'
                | '\u{061c}'
                | '\u{feff}'
        )
}

/// The characters of `text` that [`is_hidden`] does not name.
pub fn visible(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars().filter(|character| !is_hidden(*character))
}

/// `text` without its hidden characters, on one line.
pub fn plain(text: &str) -> OutsideText {
    OutsideText::from_visible(visible(text).collect())
}

/// [`plain`], cut to its first `max` characters.
pub fn plain_up_to(text: &str, max: usize) -> OutsideText {
    OutsideText::from_visible(visible(text).take(max).collect())
}
