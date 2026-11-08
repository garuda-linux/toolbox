use super::{Key, KeyMode, Modifiers, encode_key, encode_paste};

const NORMAL: KeyMode = KeyMode {
    application_cursor: false,
    bracketed_paste: false,
};

const APPLICATION: KeyMode = KeyMode {
    application_cursor: true,
    bracketed_paste: true,
};

fn plain(key: Key) -> Vec<u8> {
    encode_key(key, Modifiers::NONE, NORMAL)
}

#[test]
fn printable_characters_are_sent_as_utf8() {
    assert_eq!(plain(Key::Char('a')), b"a");
    assert_eq!(plain(Key::Char('A')), b"A");
    assert_eq!(plain(Key::Char('ü')), "ü".as_bytes());
    assert_eq!(plain(Key::Char('日')), "日".as_bytes());
}

#[test]
fn editing_keys_send_their_control_codes() {
    for (key, expected) in [
        (Key::Enter, &b"\r"[..]),
        (Key::Backspace, b"\x7f"),
        (Key::Tab, b"\t"),
        (Key::Escape, b"\x1b"),
        (Key::Insert, b"\x1b[2~"),
        (Key::Delete, b"\x1b[3~"),
        (Key::PageUp, b"\x1b[5~"),
        (Key::PageDown, b"\x1b[6~"),
    ] {
        assert_eq!(plain(key), expected, "{key:?}");
    }
}

#[test]
fn cursor_keys_follow_the_application_cursor_mode() {
    for (key, normal, application) in [
        (Key::Up, &b"\x1b[A"[..], &b"\x1bOA"[..]),
        (Key::Down, b"\x1b[B", b"\x1bOB"),
        (Key::Right, b"\x1b[C", b"\x1bOC"),
        (Key::Left, b"\x1b[D", b"\x1bOD"),
        (Key::Home, b"\x1b[H", b"\x1bOH"),
        (Key::End, b"\x1b[F", b"\x1bOF"),
    ] {
        assert_eq!(plain(key), normal, "{key:?}");
        assert_eq!(
            encode_key(key, Modifiers::NONE, APPLICATION),
            application,
            "{key:?}"
        );
    }
}

#[test]
fn modified_cursor_keys_carry_the_modifier_parameter() {
    assert_eq!(
        encode_key(Key::Up, Modifiers::SHIFT, APPLICATION),
        b"\x1b[1;2A"
    );
    assert_eq!(
        encode_key(Key::Right, Modifiers::CONTROL, NORMAL),
        b"\x1b[1;5C"
    );
    assert_eq!(
        encode_key(Key::Delete, Modifiers::ALT, NORMAL),
        b"\x1b[3;3~"
    );
}

#[test]
fn function_keys_one_to_twelve() {
    let expected: [&[u8]; 12] = [
        b"\x1bOP",
        b"\x1bOQ",
        b"\x1bOR",
        b"\x1bOS",
        b"\x1b[15~",
        b"\x1b[17~",
        b"\x1b[18~",
        b"\x1b[19~",
        b"\x1b[20~",
        b"\x1b[21~",
        b"\x1b[23~",
        b"\x1b[24~",
    ];

    for (number, bytes) in (1_u8..=12_u8).zip(expected) {
        assert_eq!(plain(Key::Function(number)), bytes, "F{number}");
    }

    assert_eq!(
        encode_key(Key::Function(1), Modifiers::SHIFT, NORMAL),
        b"\x1b[1;2P"
    );
    assert_eq!(
        encode_key(Key::Function(5), Modifiers::CONTROL, NORMAL),
        b"\x1b[15;5~"
    );
    assert_eq!(plain(Key::Function(0)), b"");
    assert_eq!(plain(Key::Function(13)), b"");
}

#[test]
fn control_turns_letters_into_control_codes() {
    let control = |character| encode_key(Key::Char(character), Modifiers::CONTROL, NORMAL);

    assert_eq!(control('c'), b"\x03");
    assert_eq!(control('C'), b"\x03");
    assert_eq!(control('a'), b"\x01");
    assert_eq!(control('z'), b"\x1a");
    assert_eq!(control('['), b"\x1b");
    assert_eq!(control(' '), b"\x00");
    assert_eq!(control('?'), b"\x7f");
    assert_eq!(control('1'), b"1");
}

#[test]
fn alt_prefixes_escape() {
    let alt = |key| encode_key(key, Modifiers::ALT, NORMAL);

    assert_eq!(alt(Key::Char('x')), b"\x1bx");
    assert_eq!(alt(Key::Enter), b"\x1b\r");
    assert_eq!(alt(Key::Backspace), b"\x1b\x7f");
    assert_eq!(
        encode_key(
            Key::Char('c'),
            Modifiers {
                control: true,
                alt: true,
                shift: false
            },
            NORMAL
        ),
        b"\x1b\x03"
    );
}

#[test]
fn shift_tab_and_control_backspace() {
    assert_eq!(encode_key(Key::Tab, Modifiers::SHIFT, NORMAL), b"\x1b[Z");
    assert_eq!(
        encode_key(Key::Backspace, Modifiers::CONTROL, NORMAL),
        b"\x08"
    );
}

#[test]
fn pasted_text_is_bracketed_when_the_program_asked() {
    assert_eq!(
        encode_paste("one\ntwo", APPLICATION),
        b"\x1b[200~one\ntwo\x1b[201~"
    );
}

#[test]
fn bracketed_paste_drops_escapes_that_could_end_it_early() {
    assert_eq!(
        encode_paste("a\x1b[201~rm", APPLICATION),
        b"\x1b[200~a[201~rm\x1b[201~"
    );
}

#[test]
fn unbracketed_paste_sends_returns_like_typed_lines() {
    assert_eq!(
        encode_paste("one\r\ntwo\nthree", NORMAL),
        b"one\rtwo\rthree"
    );
}

/// Whoever filled the clipboard chose the text. A control character in
/// it would act as a key: Ctrl+C, Ctrl+D, Ctrl+Z, an escape sequence.
#[test]
fn pasted_control_characters_are_dropped_in_both_modes() {
    let pasted = "a\x03b\x04c\x1a\x1b[2Jd\x00\x7f\u{9b}e\tf";

    assert_eq!(
        encode_paste(pasted, APPLICATION),
        b"\x1b[200~abc[2Jde\tf\x1b[201~"
    );
    assert_eq!(encode_paste(pasted, NORMAL), b"abc[2Jde\tf");
}

#[test]
fn pasted_line_breaks_stay_in_both_modes() {
    assert_eq!(
        encode_paste("one\r\ntwo\nthree\rfour", APPLICATION),
        b"\x1b[200~one\r\ntwo\nthree\rfour\x1b[201~"
    );
    assert_eq!(
        encode_paste("one\r\ntwo\nthree\rfour", NORMAL),
        b"one\rtwo\rthree\rfour"
    );
}
