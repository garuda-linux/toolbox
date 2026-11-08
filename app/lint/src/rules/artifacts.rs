//! `no-llm-artifacts`: placeholders a code generator leaves behind. Rust stubs
//! (`todo!`, `unimplemented!`, `panic!`) are clippy's, so only comments there.

use std::sync::LazyLock;

use regex::Regex;

use super::excerpt;
use crate::finding::Finding;
use crate::pattern::re;
use crate::source::Source;

pub(super) const RULE: &str = "no-llm-artifacts";

static ARTIFACT: LazyLock<Regex> = LazyLock::new(|| {
    re(concat!(
        r"(?i)\.\.\.\s*(existing|rest of|previous|other|remaining)\b",
        r"|\b(remains?|stays?) (the )?(same|unchanged)\b",
        r"|\bTODO:?\s*implement\b",
        r"|\byour (code|logic|implementation) (goes )?here\b",
        r"|\bomitted for brevity\b",
        r"|\bsame as (above|before)\b",
        r"|\badd (more )?\w+ as needed\b",
        r"|\bcontinue from here\b",
        r"|\bsee implementation above\b",
        r"|\bexisting code\b",
    ))
});

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for comment in source.comments.iter().filter(|comment| !comment.directive) {
        let text = comment.text();
        if ARTIFACT.is_match(&text) {
            findings.push(
                Finding::new(
                    comment.start(),
                    RULE,
                    format!("Placeholder comment \"{}\".", excerpt(&text)),
                    "Elision markers mean code was skipped or truncated. The file may be incomplete.",
                )
                .fix("Write the missing code or delete the comment."),
            );
        }
    }
}
