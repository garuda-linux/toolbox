//! `comment-style`: `//` comments, with `///` and `//!` for docs. Plain
//! block comments are never needed.

use crate::finding::Finding;
use crate::source::{Kind, Source};

pub(super) const RULE: &str = "comment-style";

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for comment in source
        .comments
        .iter()
        .filter(|comment| !comment.slashes && !comment.directive)
    {
        let fix = match comment.kind {
            Kind::Block => "Write it as `//` lines.",
            Kind::Doc => "Write it as `///` lines.",
            Kind::InnerDoc => "Write it as `//!` lines.",
            Kind::Line => continue,
        };

        findings.push(
            Finding::new(
                comment.start(),
                RULE,
                "Block comment.".to_owned(),
                "One comment style per purpose, so the style says nothing on its own and tools treat every comment alike.",
            )
            .fix(fix),
        );
    }
}
