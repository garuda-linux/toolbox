//! `no-filler-comments`: padding phrases ("This method…", "simply", "note
//! that") and section dividers or labels that structure code with comments
//! instead of with files and names.

use std::sync::LazyLock;

use regex::Regex;

use super::{excerpt, flatten};
use crate::finding::Finding;
use crate::pattern::re;
use crate::source::Source;

pub(super) const RULE: &str = "no-filler-comments";

static FILLER: LazyLock<Regex> = LazyLock::new(|| {
    re(concat!(
        r"(?i)^(this|the following) (function|method|class|component|service|helper|code|block|line|variable|constant|property|file|module|struct|enum|trait|type)\b",
        r"|\bhelper (function |method )?(to|that|for)\b",
        r"|\b(simply|basically|essentially|obviously|clearly|actually)\b",
        r"|\bnote that\b",
        r"|\bit(?: is|'s) (important|worth noting|crucial|essential)\b",
        r"|\bin order to\b",
        r"|\bwe (need|want|have) to\b",
        r"|^here we\b",
        r"|\bas (mentioned|described|shown) (above|below|earlier)\b",
    ))
});
static DIVIDER: LazyLock<Regex> =
    LazyLock::new(|| re(r"^[-=*#_~+.\s/]{3,}$|^[-=*#_~]{3,}.*[-=*#_~]{3,}$"));
static LABEL: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)^(imports?|exports?|constructor|getters?|setters?|(public |private |protected )?methods?|properties|fields|variables|constants|helpers?( functions)?|utils|utilities|inputs?|outputs?|state|types|interfaces|handlers|event handlers|dependencies|services|init|initiali[sz]ation|public api|public|private|tests?)\s*:?$",
    )
});
static QUOTED: LazyLock<Regex> = LazyLock::new(|| re(r#""[^"]*"|`[^`]*`"#));

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for comment in source.comments.iter().filter(|comment| !comment.directive) {
        let flat = flatten(&comment.text());
        let text = flat.as_str();

        if DIVIDER.is_match(text) || LABEL.is_match(text) {
            findings.push(
                Finding::new(
                    comment.start(),
                    RULE,
                    format!("Section divider or label comment: \"{}\".", excerpt(text)),
                    "Dividers and labels like \"// Helpers\" structure a file that is too big. Names and files should carry the structure.",
                )
                .fix("Delete it. If the file needs sections, split it into smaller files along those sections."),
            );
            continue;
        }

        if FILLER.is_match(&QUOTED.replace_all(text, "")) {
            findings.push(
                Finding::new(
                    comment.start(),
                    RULE,
                    format!("Comment uses filler phrasing: \"{}\".", excerpt(text)),
                    "Phrases like \"This method\", \"simply\", \"note that\" and \"in order to\" pad comments without adding information.",
                )
                .fix("Delete the filler and keep only the fact. If nothing is left, delete the comment."),
            );
        }
    }
}
