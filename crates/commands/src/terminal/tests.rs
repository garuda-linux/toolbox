use super::{Typed, read_secret};

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
