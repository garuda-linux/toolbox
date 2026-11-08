use super::count;

const RULE: &str = "blank-before-comment";

#[test]
fn comment_touching_code_above_is_reported() {
    let text = "fn f() {\n    let a = 1;\n    // The second value waits for the first.\n    let b = a;\n}\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn comment_after_blank_line_passes() {
    let text = "fn f() {\n    let a = 1;\n\n    // The second value waits for the first.\n    let b = a;\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_at_block_start_passes() {
    let text = "fn f() {\n    // Nothing ran before this point.\n    go();\n}\n";
    assert_eq!(count(RULE, text), 0);

    let call = "fn f() {\n    go(\n        // The first argument is the target.\n        target,\n    );\n}\n";
    assert_eq!(count(RULE, call), 0);
}

#[test]
fn comment_at_file_start_passes() {
    let text = "// The root of the crate.\nfn f() {}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn run_of_lines_is_reported_once() {
    let text = "fn f() {\n    go();\n    // One reason\n    // across two lines.\n    stop();\n}\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn trailing_comment_passes() {
    let text = "fn f() {\n    go();\n    stop(); // The run ends here.\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn doc_comments_pass() {
    let text = "struct S {\n    a: u8,\n    /// The second field.\n    b: u8,\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_under_other_comment_passes() {
    let text = "fn f() {\n    go();\n\n    // toolbox-lint: allow(no-filler-comments) The word is a name here.\n    // Basically the default.\n    stop();\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_in_method_chain_passes() {
    let text = "fn f() {\n    build()\n        .flag(1)\n        // The second flag needs the first.\n        .flag(2);\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_between_list_elements_passes() {
    let text = "fn f() {\n    let args = [\n        \"0\",\n        // Hints: the desktop entry names the app.\n        \"1\",\n    ];\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_between_match_arms_passes() {
    let text = "fn f(x: u8) -> u8 {\n    match x {\n        0 => 1,\n        // Everything else counts as one.\n        _ => 2,\n    }\n}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn comment_between_items_is_reported() {
    let text = "const A: u8 = 1;\n// The second constant follows the first.\nconst B: u8 = A;\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn comment_under_attribute_passes() {
    let text = "fn f() {\n    go();\n\n    #[expect(unsafe_code, reason = \"one call\")]\n    // SAFETY: nothing else runs yet.\n    unsafe { start() };\n}\n";
    assert_eq!(count(RULE, text), 0);

    let multiline = "#[expect(\n    unsafe_code,\n    reason = \"one call\"\n)]\n// SAFETY: nothing else runs yet.\nfn f() {}\n";
    assert_eq!(count(RULE, multiline), 0);
}

#[test]
fn block_comment_is_reported() {
    let text = "fn f() {\n    go();\n    /* The run ends here. */\n    stop();\n}\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn finding_points_at_the_comment() {
    let findings = crate::rules::lint_text(
        "fn f() {\n    let a = 1;\n    // The second value waits for the first.\n    let b = a;\n}\n",
    );

    let hit = findings.iter().find(|finding| finding.rule == RULE);
    assert_eq!(
        hit.map(|finding| (finding.at.row, finding.at.column)),
        Some((2, 4))
    );
}
