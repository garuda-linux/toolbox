use super::count;

const RULE: &str = "no-comment-semicolon";

#[test]
fn semicolons_in_prose_are_reported() {
    assert_eq!(
        count(
            RULE,
            "// Rounds per line; the total is derived.\nfn f() {}\n"
        ),
        1
    );
    assert_eq!(
        count(
            RULE,
            "/// Loads the draft; falls back; never throws.\nfn f() {}\n"
        ),
        2
    );
}

#[test]
fn code_in_comments_passes() {
    let text = "/// Call `load(); render();` in order.\n///\n/// ```\n/// a(); b();\n/// ```\nfn f() {}\n// let total = sum(&lines);\nfn g() {}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn findings_point_at_the_semicolon() {
    let findings = crate::rules::lint_text("fn f() {}\n    // One; two.\n");
    let semicolon = findings.iter().find(|finding| finding.rule == RULE);
    assert_eq!(
        semicolon.map(|finding| (finding.at.row, finding.at.column)),
        Some((1, 10))
    );
}
