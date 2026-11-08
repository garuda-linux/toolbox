use super::count;

const RULE: &str = "separate-statements";

#[test]
fn single_line_statements_pass() {
    let text = "fn f() {\n    let a = 1;\n    let b = 2;\n    let c = a + b;\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn multiline_statement_without_blank_line_is_reported() {
    let text = "fn f() {\n    let map = data\n        .as_object()?;\n    let next = 1;\n}\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn multiline_statement_with_blank_line_passes() {
    let text = "fn f() {\n    let map = data\n        .as_object()?;\n\n    let next = 1;\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_between_passes() {
    let text = "fn f() {\n    let map = data\n        .as_object()?;\n    // The entries follow.\n    let next = 1;\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn only_a_multiline_previous_statement_counts() {
    let short_then_long =
        "fn f() {\n    let a = 1;\n    let map = data\n        .as_object()?;\n}\n";

    assert_eq!(count(RULE, short_then_long), 0);
}

#[test]
fn builder_chains_ending_in_semicolon_pass() {
    let call_then_let = "fn f() {\n    entries.sort_by(|left, right| {\n        left.cmp(right)\n    });\n    let index = 1;\n}\n";

    assert_eq!(count(RULE, call_then_let), 0);
}

#[test]
fn closing_block_without_blank_line_is_reported() {
    let text = "fn f() {\n    if ready {\n        go();\n    }\n    let next = 1;\n}\n";
    assert_eq!(count(RULE, text), 1);
    let loops = "fn f() {\n    for item in items {\n        take(item);\n    }\n    done();\n}\n";
    assert_eq!(count(RULE, loops), 1);
    let matched = "fn f() {\n    match code {\n        0 => ok(),\n        _ => err(),\n    }\n    cleanup();\n}\n";
    assert_eq!(count(RULE, matched), 1);
}

#[test]
fn closing_block_with_blank_line_passes() {
    let text = "fn f() {\n    if ready {\n        go();\n    }\n\n    let next = 1;\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn single_line_block_passes() {
    let text = "fn f() {\n    if ready { go(); }\n    let next = 1;\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn block_problem_names_the_block() {
    let findings = crate::rules::lint_text(
        "fn f() {\n    if ready {\n        go();\n    }\n    let next = 1;\n}\n",
    );

    let hit = findings.iter().find(|finding| finding.rule == RULE);
    assert_eq!(
        hit.map(|finding| finding.problem.as_str()),
        Some("Multiline block touches the next statement.")
    );
}

#[test]
fn multiline_let_before_any_statement_counts() {
    let text = "fn f() {\n    let entries = items\n        .iter()\n        .collect::<Vec<_>>();\n    entries.sort();\n}\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn findings_point_at_the_next_statement() {
    let findings = crate::rules::lint_text(
        "fn f() {\n    let map = data\n        .as_object()?;\n    let next = 1;\n}\n",
    );

    let hit = findings.iter().find(|finding| finding.rule == RULE);
    assert_eq!(
        hit.map(|finding| (finding.at.row, finding.at.column)),
        Some((3, 4))
    );
}
