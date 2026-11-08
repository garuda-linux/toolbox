//! The rules. Each module checks one rule and reports under its name.

mod artifacts;
mod blank_before;
mod comment_lines;
mod comment_style;
mod commented_code;
mod doc_restating;
mod filler;
mod restating;
mod semicolon;
mod separate;
#[cfg(test)]
mod tests;
mod unparsed;
mod variant_label;

use tree_sitter::Tree;

use crate::finding::{Finding, apply_allows};
use crate::source::{Source, parse};

/// Every name an allow directive may give: each rule's own and the two
/// the directives themselves report under.
pub const NAMES: [&str; 14] = [
    artifacts::RULE,
    blank_before::RULE,
    comment_lines::RULE,
    comment_style::RULE,
    commented_code::RULE,
    doc_restating::RULE,
    filler::RULE,
    restating::RULE,
    semicolon::RULE,
    separate::RULE,
    unparsed::RULE,
    variant_label::RULE,
    crate::finding::WITHOUT_REASON,
    crate::finding::UNKNOWN_RULE,
];

/// The most characters of comment text a message quotes.
const EXCERPT_CHARS: usize = 50;
const ELLIPSIS: &str = "...";

/// Every finding in `text`, after allow directives.
pub fn lint_text(text: &str) -> Vec<Finding> {
    lint_tree(text, parse(text).as_ref())
}

fn lint_tree(text: &str, tree: Option<&Tree>) -> Vec<Finding> {
    let Some(tree) = tree else {
        return vec![unparsed::missing()];
    };

    let source = Source::new(text, tree);
    let mut findings = Vec::new();

    artifacts::check(&source, &mut findings);
    blank_before::check(&source, &mut findings);
    comment_lines::check(&source, &mut findings);
    comment_style::check(&source, &mut findings);
    commented_code::check(&source, &mut findings);
    doc_restating::check(&source, &mut findings);
    filler::check(&source, &mut findings);
    restating::check(&source, &mut findings);
    semicolon::check(&source, &mut findings);
    separate::check(&source, &mut findings);
    variant_label::check(&source, &mut findings);

    apply_allows(&source, findings)
}

/// Comment text on one line, every run of blanks one space.
fn flatten(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn excerpt(text: &str) -> String {
    let flat = flatten(text);
    if flat.chars().count() <= EXCERPT_CHARS {
        return flat;
    }

    let short: String = flat.chars().take(EXCERPT_CHARS - ELLIPSIS.len()).collect();
    format!("{short}{ELLIPSIS}")
}
