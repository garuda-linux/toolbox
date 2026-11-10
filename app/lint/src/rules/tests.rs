mod artifacts;
mod blank_before;
mod comment_lines;
mod comment_style;
mod commented_code;
mod doc_restating;
mod domain_registered;
mod filler;
mod no_shell_command;
mod program_path;
mod restating;
mod semicolon;
mod separate;
mod sudo_once;
mod suppression;
mod tests_in_file;
mod variant_label;

use std::path::Path;

use super::{lint_file, lint_text};
use crate::source::Place;

fn rules(text: &str) -> Vec<&'static str> {
    lint_text(text)
        .into_iter()
        .map(|finding| finding.rule)
        .collect()
}

fn count(rule: &str, text: &str) -> usize {
    rules(text)
        .into_iter()
        .filter(|found| *found == rule)
        .count()
}

/// Findings of `rule` in `text` as if it were the file at `path`.
fn count_at(rule: &str, path: &str, text: &str) -> usize {
    lint_file(Path::new(path), text)
        .into_iter()
        .filter(|finding| finding.rule == rule)
        .count()
}

#[test]
fn a_missing_tree_is_reported_as_unparsed() {
    let place = Place::new(Path::new("crates/x/src/lib.rs"));
    let findings = super::lint_tree(&place, "", None, None);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, "unparsed");
}

#[test]
fn broken_code_parses_into_errors_not_into_unparsed() {
    let findings = lint_text("fn f( { !! }\n");

    assert!(
        findings.iter().all(|finding| finding.rule != "unparsed"),
        "error nodes are no missing tree"
    );
}
