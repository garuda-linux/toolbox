use super::count;

const RULE: &str = "comment-style";

#[test]
fn block_comments_are_reported() {
    assert_eq!(count(RULE, "/* Why. */\nfn f() {}\n"), 1);
    assert_eq!(count(RULE, "/** Docs. */\nfn f() {}\n"), 1);
}

#[test]
fn line_comments_pass() {
    assert_eq!(count(RULE, "// Why.\n/// Docs.\nfn f() {}\n"), 0);
}
