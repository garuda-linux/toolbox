use super::count;

const RULE: &str = "no-commented-out-code";

#[test]
fn commented_out_rust_is_reported_once_per_run() {
    let text = "fn f() {\n    // let total = lines.iter().sum::<u32>();\n    // if total > 3 {\n    //     warn(total);\n    // }\n}\n";
    assert_eq!(count(RULE, text), 1);
    assert_eq!(count(RULE, "/* self.load(); */\nfn f() {}\n"), 1);
}

#[test]
fn prose_markers_and_docs_pass() {
    let text = "// Lines round per line, not per document\n// Year\n// TODO: remove after migration\n// https://example.org/x\n/// `let x = 1;` in a doc\nfn f() {}\n";
    assert_eq!(count(RULE, text), 0);
}

#[test]
fn word_lists_are_prose() {
    assert_eq!(
        count(
            RULE,
            "// neutral, positive, warning, negative, info\nconst TONE: u8 = 0;\n"
        ),
        0
    );
}
