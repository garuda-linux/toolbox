use super::count;

const RULE: &str = "no-llm-artifacts";

#[test]
fn placeholder_comments_are_reported() {
    for comment in [
        "// ... existing code",
        "// rest of the function stays the same",
        "// TODO: implement",
        "// your logic goes here",
        "// omitted for brevity",
        "// same as above",
    ] {
        assert_eq!(
            count(RULE, &format!("{comment}\nfn f() {{}}\n")),
            1,
            "{comment}"
        );
    }
}

#[test]
fn ordinary_comments_pass() {
    let text = "// The rest of an escape sequence after ESC.\nfn f() {}\n";
    assert_eq!(count(RULE, text), 0);
}
