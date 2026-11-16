use std::io::Write as _;
use std::os::fd::AsFd as _;

use super::{LINE_BYTES, Typed, read_line_from, read_secret};

fn typed(bytes: &[u8]) -> Typed {
    read_secret(&mut &*bytes)
}

#[test]
fn a_line_ends_with_return_or_a_line_feed() {
    assert_eq!(typed(b"hunter2\r"), Typed::Line("hunter2".to_owned()));
    assert_eq!(
        typed(b"hunter2\nignored"),
        Typed::Line("hunter2".to_owned())
    );
}

#[test]
fn backspace_takes_back_the_last_character() {
    assert_eq!(typed(b"ab\x7fc\r"), Typed::Line("ac".to_owned()));
    assert_eq!(
        typed("pä\x08ss\r".as_bytes()),
        Typed::Line("pss".to_owned())
    );
}

#[test]
fn control_c_cancels_and_control_d_on_an_empty_line_closes() {
    assert_eq!(typed(b"secr\x03et\r"), Typed::Cancelled);
    assert_eq!(typed(b"\x04"), Typed::Closed);
    assert_eq!(typed(b""), Typed::Closed);
}

#[test]
fn input_that_ends_without_return_is_closed_not_taken() {
    assert_eq!(typed(b"half"), Typed::Closed);
}

fn piped(bytes: &[u8]) -> std::io::PipeReader {
    let (reader, mut writer) = std::io::pipe().unwrap();

    writer.write_all(bytes).unwrap();
    drop(writer);
    reader
}

/// Nothing past the line is read, so the next reader still gets it.
#[test]
fn the_first_stdin_line_leaves_the_rest_of_the_input_unread() {
    let reader = piped(b"one\ntwo\n");

    assert_eq!(read_line_from(reader.as_fd()), Some("one".to_owned()));
    assert_eq!(read_line_from(reader.as_fd()), Some("two".to_owned()));
    assert_eq!(read_line_from(reader.as_fd()), None);
}

#[test]
fn a_stdin_line_of_the_password_room_is_taken_and_a_longer_one_refused() {
    let fits = format!("{}\n", "a".repeat(LINE_BYTES));
    let too_long = format!("{}\n", "a".repeat(LINE_BYTES + 1));

    assert_eq!(
        read_line_from(piped(fits.as_bytes()).as_fd()),
        Some("a".repeat(LINE_BYTES))
    );
    assert_eq!(read_line_from(piped(too_long.as_bytes()).as_fd()), None);
}

#[test]
fn a_stdin_line_without_its_line_end_is_taken_at_the_end_of_input() {
    assert_eq!(
        read_line_from(piped(b"half").as_fd()),
        Some("half".to_owned())
    );
    assert_eq!(read_line_from(piped(b"").as_fd()), None);
}
