use super::{OutsideText, is_hidden, plain, plain_up_to, visible};

#[test]
fn controls_and_escapes_are_hidden() {
    assert_eq!(plain("a\u{1b}[31mred\u{7}\tb"), "a[31mredb");
}

/// Each of these reorders, hides or breaks a line of text instead of
/// showing a character, so a name could look like another one.
#[test]
fn invisible_and_reordering_characters_are_hidden() {
    for character in [
        '\u{202e}', '\u{2066}', '\u{2069}', '\u{200e}', '\u{200f}', '\u{061c}', '\u{200b}',
        '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}', '\u{2028}', '\u{2029}',
    ] {
        assert!(is_hidden(character), "{:04x}", u32::from(character));
    }
}

#[test]
fn letters_of_any_script_stay() {
    assert_eq!(plain("pâte über ж 中文 😀"), "pâte über ж 中文 😀");
}

#[test]
fn plain_up_to_counts_visible_characters_only() {
    assert_eq!(plain_up_to("a\u{202e}bc\u{7}de", 3), "abc");
    assert_eq!(plain_up_to("äöüß", 2), "äö");
}

#[test]
fn visible_skips_the_hidden_characters() {
    assert_eq!(visible("x\u{200b}y\n").collect::<String>(), "xy");
}

#[test]
fn keep_lines_keeps_line_breaks_and_tabs() {
    assert_eq!(
        OutsideText::keep_lines("a\tb\r\n\u{1b}[1mc\u{202e}\n"),
        "a\tb\n[1mc\n"
    );
}

#[test]
fn outside_text_serializes_as_a_plain_string() {
    let text = plain("pkg\u{7}");

    assert_eq!(serde_json::to_string(&text).unwrap(), r#""pkg""#);
}
