//! Text from outside (a server's answer, a package description, a feed)
//! shown as it is, without what could change how it reads.

#[cfg(test)]
mod tests;

/// Whether `character` must not reach the display.
///
/// Controls (ESC among them), bidi overrides, isolates and marks,
/// zero-width characters, the byte order mark and the line and paragraph
/// separators each change how the text around them reads instead of
/// showing themselves.
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

/// `text` without its [hidden](is_hidden) characters.
pub fn plain(text: &str) -> String {
    text.chars()
        .filter(|character| !is_hidden(*character))
        .collect()
}
