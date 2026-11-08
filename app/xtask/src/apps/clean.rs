//! Wiki markup to plain text, and website links the catalog accepts. The
//! cards show plain text only, so no template, link or quote markup may
//! survive.

use std::sync::LazyLock;

use regex::Regex;
use toolbox_catalog::is_web_url;

use crate::regex;

/// Applied in order: a later rule relies on the earlier ones having
/// removed their templates.
const RULES_SOURCE: &[(&str, &str)] = &[
    (r"(?i)\{\{Grp\|kde-games\}\}", "Part of KDE Games"),
    (r"(?i)\{\{Dead link\|.*?\}\}", ""),
    (r"(?i)\{\{man\|.*?\|(.*?)(?:\|.*?)*\}\}", "$1"),
    (r"(?i)\{\{Pkg\|(.*?)(?:\|.*?)*\}\}", "$1"),
    (r"(?i)\{\{AUR\|(.*?)(?:\|.*?)*\}\}", "$1"),
    (r"\{\{=\}\}", "="),
    (r"(?i)\{\{ic\|(.*?)\}\}", "$1"),
    (r"(?i)\{\{Grp\|(.*?)\}\}", "$1"),
    (r"\{\{.*?\}\}", ""),
    (r"\[\[Wikipedia:(.*?)\|(.*?)\]\]", "$2"),
    (r"\[\[(.*?)\|(.*?)\]\]", "$2"),
    (r"\[\[(.*?)\]\]", "$1"),
    (r"\[https?://[^\s\]]*\]", ""),
    (r"\[https?://.*?\s+(.*?)\]", "$1"),
    ("'''", ""),
    ("''", ""),
    ("&amp;", "&"),
    ("&nbsp;", " "),
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    (r"\s+", " "),
    (r"\s+([.,;:])(\s|$)", "$1$2"),
    (r"([^.])\.\.$", "$1."),
];

static RULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    RULES_SOURCE
        .iter()
        .map(|(pattern, with)| (regex(pattern), *with))
        .collect()
});

pub fn wiki_text(text: &str) -> String {
    let mut plain = text.trim().to_owned();

    for (regex, with) in RULES.iter() {
        plain = regex.replace_all(&plain, *with).into_owned();
    }

    plain.trim().to_owned()
}

/// The first link in `raw`, when it is one the catalog accepts. The wiki
/// sometimes lists two sites in one field, and plain `http` sites are left
/// out: the app only opens `https`.
pub fn https_url(raw: &str) -> Option<String> {
    raw.split_whitespace()
        .next()
        .filter(|link| is_web_url(link))
        .map(ToOwned::to_owned)
}
