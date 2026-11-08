use super::{ENTRY_CHARS, LIST_ENTRIES, list, optional, plain};

#[test]
fn controls_escapes_and_bidi_overrides_are_dropped() {
    assert_eq!(
        plain("a\u{1b}[31mb\nc\u{202e}d\u{2066}e\u{200f}f\u{7}", 100),
        "a[31mbcdef"
    );
}

#[test]
fn text_is_cut_by_characters() {
    assert_eq!(plain("äöüß", 2), "äö");
}

#[test]
fn a_blank_optional_value_is_none() {
    assert_eq!(optional(Some(" \u{1b} "), 10), None);
    assert_eq!(optional(None, 10), None);
    assert_eq!(optional(Some("ada"), 10), Some("ada".to_owned()));
}

#[test]
fn a_list_is_bounded_in_entries_and_length() {
    let long = "x".repeat(ENTRY_CHARS + 10);
    let entries: Vec<String> = std::iter::repeat_n(long, LIST_ENTRIES + 5).collect();

    let kept = list(&entries);

    assert_eq!(kept.len(), LIST_ENTRIES);
    assert!(
        kept.iter()
            .all(|entry| entry.chars().count() == ENTRY_CHARS)
    );
}

#[test]
fn empty_list_entries_are_left_out() {
    let entries = vec![String::new(), "\u{1b}".to_owned(), "glibc".to_owned()];

    assert_eq!(list(&entries), ["glibc"]);
}
