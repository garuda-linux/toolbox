//! The rules. Each module checks one rule and reports under its name.

mod artifacts;
mod blank_before;
mod comment_lines;
mod comment_style;
mod commented_code;
mod doc_restating;
pub mod domain_registered;
mod filler;
mod no_shell_command;
mod program_path;
mod restating;
mod semicolon;
mod separate;
mod sudo_once;
#[cfg(test)]
mod tests;
mod tests_in_file;
mod unparsed;
mod variant_label;

use std::collections::BTreeSet;
use std::path::Path;

use tree_sitter::Tree;

use crate::finding::{Finding, apply_allows};
use crate::source::{Place, Source, parse};

/// Every name an allow directive may give: each rule's own and the two
/// the directives themselves report under.
pub const NAMES: [&str; 19] = [
    artifacts::RULE,
    blank_before::RULE,
    comment_lines::RULE,
    comment_style::RULE,
    commented_code::RULE,
    doc_restating::RULE,
    domain_registered::RULE,
    filler::RULE,
    no_shell_command::RULE,
    program_path::RULE,
    restating::RULE,
    semicolon::RULE,
    separate::RULE,
    sudo_once::RULE,
    tests_in_file::RULE,
    unparsed::RULE,
    variant_label::RULE,
    crate::finding::WITHOUT_REASON,
    crate::finding::UNKNOWN_RULE,
];

/// The most characters of comment text a message quotes.
const EXCERPT_CHARS: usize = 50;
const ELLIPSIS: &str = "...";

/// Every finding in `text`, the file at `path`, after allow directives.
#[cfg(test)]
pub fn lint_file(path: &Path, text: &str) -> Vec<Finding> {
    lint_file_in(path, text, None)
}

/// [`lint_file`] in a tree whose `domains::all()` registers `domains`,
/// `None` when the tree has no registry.
pub fn lint_file_in(path: &Path, text: &str, domains: Option<&BTreeSet<String>>) -> Vec<Finding> {
    lint_tree(&Place::new(path), text, parse(text).as_ref(), domains)
}

/// The findings of a file in no folder the place rules know.
#[cfg(test)]
pub fn lint_text(text: &str) -> Vec<Finding> {
    lint_file(Path::new("lib.rs"), text)
}

fn lint_tree(
    place: &Place<'_>,
    text: &str,
    tree: Option<&Tree>,
    domains: Option<&BTreeSet<String>>,
) -> Vec<Finding> {
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
    domain_registered::check(&source, place, domains, &mut findings);
    filler::check(&source, &mut findings);
    no_shell_command::check(&source, place, &mut findings);
    program_path::check(&source, place, &mut findings);
    restating::check(&source, &mut findings);
    semicolon::check(&source, &mut findings);
    separate::check(&source, &mut findings);
    sudo_once::check(&source, place, &mut findings);
    tests_in_file::check(&source, place, &mut findings);
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
