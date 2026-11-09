use egui::TextBuffer as _;
use egui::epaint::text::CharIndex;

use super::SecretText;

#[test]
fn typing_fills_the_room_and_never_grows_it() {
    let mut text = SecretText::with_room(8);
    let room = text.0.capacity();

    assert_eq!(text.insert_text("correct horse", CharIndex(0)), room);
    assert_eq!(text.as_str().len(), room);
    assert_eq!(text.0.capacity(), room, "the buffer was reallocated");
}

#[test]
fn a_character_that_does_not_fit_whole_is_left_out() {
    let mut text = SecretText::with_room(8);
    let room = text.0.capacity();
    let filler = "a".repeat(room - 1);

    text.insert_text(&filler, CharIndex(0));

    assert_eq!(text.insert_text("ü", CharIndex(room - 1)), 0);
    assert_eq!(text.as_str(), filler);
}

#[test]
fn edits_insert_and_delete_by_character() {
    let mut text = SecretText::with_room(64);

    text.insert_text("pässwort", CharIndex(0));
    text.insert_text("!", CharIndex(1));
    text.delete_char_range(CharIndex(2)..CharIndex(4));

    assert_eq!(text.as_str(), "p!swort");
}

#[test]
fn nothing_is_handed_out_or_shown() {
    let mut text = SecretText::with_room(64);

    text.insert_text("hunter2", CharIndex(0));

    assert_eq!(format!("{text:?}"), "SecretText(<redacted>)");
    assert_eq!(text.take(), "");
    assert_eq!(text.as_str(), "");
}
