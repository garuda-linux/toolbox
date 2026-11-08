use super::{is_hidden, plain};

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
