use super::{hidden, matches_english};

const ENGLISH: &str = "greeting = Hello { $name }\nbye = Bye\n";

#[test]
fn bidi_and_zero_width_characters_are_found() {
    assert_eq!(hidden("Neu\u{202E}starten"), Some('\u{202E}'));
    assert_eq!(hidden("Neu\u{200B}starten"), Some('\u{200B}'));
    assert_eq!(hidden("Neu\u{7}starten"), Some('\u{7}'));
}

#[test]
fn the_line_and_paragraph_separators_and_the_arabic_mark_are_found() {
    for letter in ['\u{2028}', '\u{2029}', '\u{61C}'] {
        assert_eq!(hidden(&format!("Neu{letter}starten")), Some(letter));
    }
}

#[test]
fn plain_text_line_breaks_and_tabs_pass() {
    assert_eq!(hidden("Neu starten\n\tjetzt, äöü"), None);
}

#[test]
fn a_file_with_english_ids_and_placeables_matches() {
    assert_eq!(
        matches_english("pages", ENGLISH, "greeting = Hallo { $name }\n"),
        Ok(())
    );
}

#[test]
fn a_foreign_id_does_not() {
    matches_english("pages", ENGLISH, "other = Hallo\n").unwrap_err();
}
