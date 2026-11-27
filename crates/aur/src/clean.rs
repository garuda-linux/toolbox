//! Text from the AUR as plain, bounded text. Anyone can upload a package,
//! so its texts never reach the display with control characters, escape
//! sequences or bidi overrides, and never at any length.

#[cfg(test)]
mod tests;

use toolbox_text::OutsideText;

/// The longest kept version, `epoch:pkgver-pkgrel`.
pub const VERSION_CHARS: usize = 128;
pub const DESCRIPTION_CHARS: usize = 512;
/// AUR account names are at most 16 characters.
pub const MAINTAINER_CHARS: usize = 64;
pub const URL_CHARS: usize = 2048;
/// One entry of a list (a dependency, a licence, a keyword).
const ENTRY_CHARS: usize = 256;
const LIST_ENTRIES: usize = 256;

pub fn plain(text: &str, max: usize) -> OutsideText {
    toolbox_text::plain_up_to(text, max)
}

pub fn optional(text: Option<&str>, max: usize) -> Option<OutsideText> {
    text.map(|text| plain(text, max))
        .filter(|text| !text.trim().is_empty())
}

pub fn list(entries: &[String]) -> Vec<OutsideText> {
    entries
        .iter()
        .map(|entry| plain(entry, ENTRY_CHARS))
        .filter(|entry| !entry.trim().is_empty())
        .take(LIST_ENTRIES)
        .collect()
}
