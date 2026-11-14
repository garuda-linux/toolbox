//! Transifex text is outside text. Before a translation reaches a Fluent
//! file it is screened for characters a reviewer cannot see, and the file
//! that results is parsed again and held against English.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use super::{parsed, variables};

/// The first character of `text` that hides itself or reorders what is
/// shown: controls but line breaks and tabs, bidi controls and
/// zero-width characters.
pub(super) fn hidden(text: &str) -> Option<char> {
    text.chars().find(|letter| {
        (letter.is_control() && !matches!(letter, '\n' | '\t'))
            || matches!(
                letter,
                '\u{200B}'..='\u{200F}'
                    | '\u{202A}'..='\u{202E}'
                    | '\u{2060}'..='\u{2064}'
                    | '\u{2066}'..='\u{2069}'
                    | '\u{FEFF}'
            )
    })
}

/// `written` holds only English's ids, each with no placeable English
/// lacks.
///
/// # Errors
///
/// The first message that is not, or a file that no longer parses.
pub(super) fn matches_english(stem: &str, english: &str, written: &str) -> Result<(), String> {
    let english = parsed(stem, english)?;

    for message in parsed(stem, written)? {
        let Some(source) = english.iter().find(|source| source.id == message.id) else {
            return Err(format!("{stem}.ftl: {} is no English message", message.id));
        };

        let (Ok(source), Ok(value)) = (&source.value, &message.value) else {
            continue;
        };

        let allowed: BTreeSet<String> = variables(source).into_iter().collect();
        if let Some(name) = variables(value)
            .into_iter()
            .find(|name| !allowed.contains(name))
        {
            return Err(format!(
                "{stem}.ftl: {} uses `{name}`, which the English text does not have",
                message.id
            ));
        }
    }

    Ok(())
}

/// `text`, unless it hides a character from its reviewer.
pub(super) fn screened(text: &str) -> Result<&str, String> {
    hidden(text).map_or(Ok(text), |letter| {
        Err(format!("holds the hidden character U+{:04X}", u32::from(letter)))
    })
}
