//! Text from the AUR as plain, bounded text. Anyone can upload a package,
//! so its texts never reach the display with control characters, escape
//! sequences or bidi overrides, and never at any length.

#[cfg(test)]
mod tests;

/// The longest kept version, `epoch:pkgver-pkgrel` with room to spare.
pub const VERSION_CHARS: usize = 128;
pub const DESCRIPTION_CHARS: usize = 512;
/// AUR account names are at most 16 characters.
pub const MAINTAINER_CHARS: usize = 64;
pub const URL_CHARS: usize = 2048;
/// One entry of a list (a dependency, a licence, a keyword).
pub const ENTRY_CHARS: usize = 256;
pub const LIST_ENTRIES: usize = 256;

/// `text` without hidden characters, cut to `max` characters.
pub fn plain(text: &str, max: usize) -> String {
    text.chars()
        .filter(|character| !toolbox_plan::text::is_hidden(*character))
        .take(max)
        .collect()
}

/// [`plain`] of a value that may be missing, `None` once nothing is left.
pub fn optional(text: Option<&str>, max: usize) -> Option<String> {
    text.map(|text| plain(text, max))
        .filter(|text| !text.trim().is_empty())
}

/// The first [`LIST_ENTRIES`] entries, each [`plain`], empty ones left out.
pub fn list(entries: &[String]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| plain(entry, ENTRY_CHARS))
        .filter(|entry| !entry.trim().is_empty())
        .take(LIST_ENTRIES)
        .collect()
}
