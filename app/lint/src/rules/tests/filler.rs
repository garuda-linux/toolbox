use super::count;

const RULE: &str = "no-filler-comments";

#[test]
fn filler_phrases_are_reported() {
    for comment in [
        "// This function loads the file.",
        "// Simply read the lock.",
        "// Note that the cache is shared.",
        "// It is important to keep the order.",
        "// In order to save time, cache it.",
        "// We need to drop the guard here.",
    ] {
        assert_eq!(
            count(RULE, &format!("{comment}\nfn f() {{}}\n")),
            1,
            "{comment}"
        );
    }
}

#[test]
fn dividers_and_labels_are_reported() {
    for comment in [
        "// ----------",
        "// === Helpers ===",
        "// Helpers",
        "// Private methods:",
    ] {
        assert_eq!(
            count(RULE, &format!("{comment}\nfn f() {{}}\n")),
            1,
            "{comment}"
        );
    }
}

#[test]
fn facts_and_quoted_phrases_pass() {
    let text = "// The lock wins over the cache. \"simply\" is a user word.\nfn f() {}\n";
    assert_eq!(count(RULE, text), 0);
}
