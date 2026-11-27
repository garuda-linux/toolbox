//! Transifex text is outside text. Before a translation reaches a Fluent
//! file it is screened for characters a reviewer cannot see, and the file
//! that results is parsed again and held against English.

#[cfg(test)]
mod tests;

use super::parsed;

/// The first character of `text` that hides itself or reorders what is
/// shown: controls but line breaks and tabs, bidi controls and
/// zero-width characters.
pub(super) fn hidden(text: &str) -> Option<char> {
    text.chars()
        .find(|letter| toolbox_text::is_hidden(*letter) && !matches!(letter, '\n' | '\t'))
}

/// `written` holds only English's ids.
///
/// # Errors
///
/// The first message that is not, or a file that no longer parses.
pub(super) fn matches_english(stem: &str, english: &str, written: &str) -> Result<(), String> {
    let english = parsed(stem, english)?;

    for message in parsed(stem, written)? {
        if !english.iter().any(|source| source.id == message.id) {
            return Err(format!("{stem}.ftl: {} is no English message", message.id));
        }
    }

    Ok(())
}

/// `text`, unless it hides a character from its reviewer.
pub(super) fn screened(text: &str) -> Result<&str, String> {
    hidden(text).map_or(Ok(text), |letter| {
        Err(format!(
            "holds the hidden character U+{:04X}",
            u32::from(letter)
        ))
    })
}
