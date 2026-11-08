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
mod suppression;
mod variant_label;

use super::lint_text;

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

#[test]
fn a_missing_tree_is_reported_as_unparsed() {
    let findings = super::lint_tree("", None);

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
