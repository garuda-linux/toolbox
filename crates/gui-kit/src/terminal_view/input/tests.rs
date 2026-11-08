use egui::{Event, Key, Modifiers};
use toolbox_terminal::KeyMode;

use super::{Input, translate};

const CTRL_SHIFT: Modifiers = Modifiers {
    alt: false,
    ctrl: true,
    shift: true,
    mac_cmd: false,
    command: true,
};

fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn bytes(event: &Event, held: Modifiers) -> Input {
    translate(event, held, KeyMode::default())
}

#[test]
fn typed_text_goes_to_the_program() {
    assert_eq!(
        bytes(&Event::Text("ls -l".to_owned()), Modifiers::NONE),
        Input::Bytes(b"ls -l".to_vec())
    );
}

#[test]
fn keys_that_are_not_text_are_encoded() {
    assert_eq!(
        bytes(&key(Key::Enter, Modifiers::NONE), Modifiers::NONE),
        Input::Bytes(b"\r".to_vec())
    );
    assert_eq!(
        bytes(&key(Key::ArrowUp, Modifiers::NONE), Modifiers::NONE),
        Input::Bytes(b"\x1b[A".to_vec())
    );
    assert_eq!(
        bytes(&key(Key::Tab, Modifiers::NONE), Modifiers::NONE),
        Input::Bytes(b"\t".to_vec())
    );
    assert_eq!(
        bytes(&key(Key::Escape, Modifiers::NONE), Modifiers::NONE),
        Input::Bytes(b"\x1b".to_vec())
    );
}

#[test]
fn a_plain_letter_key_waits_for_its_text() {
    assert_eq!(
        bytes(&key(Key::A, Modifiers::NONE), Modifiers::NONE),
        Input::Nothing
    );
}

#[test]
fn ctrl_and_a_letter_sends_its_control_code() {
    assert_eq!(
        bytes(&key(Key::D, Modifiers::CTRL), Modifiers::CTRL),
        Input::Bytes(vec![0x04])
    );
}

#[test]
fn ctrl_c_x_and_v_reach_the_program() {
    assert_eq!(
        bytes(&Event::Copy, Modifiers::CTRL),
        Input::Bytes(vec![0x03])
    );
    assert_eq!(
        bytes(&Event::Cut, Modifiers::CTRL),
        Input::Bytes(vec![0x18])
    );
    assert_eq!(
        bytes(&Event::Paste("ignored".to_owned()), Modifiers::CTRL),
        Input::Bytes(vec![0x16])
    );
}

#[test]
fn ctrl_shift_c_copies_and_ctrl_shift_v_asks_for_the_clipboard() {
    assert_eq!(bytes(&key(Key::C, CTRL_SHIFT), CTRL_SHIFT), Input::Copy);
    assert_eq!(bytes(&key(Key::V, CTRL_SHIFT), CTRL_SHIFT), Input::Paste);
}

#[test]
fn ctrl_tab_leaves_the_console() {
    assert_eq!(
        bytes(&key(Key::Tab, Modifiers::CTRL), Modifiers::CTRL),
        Input::Leave { backward: false }
    );
    assert_eq!(
        bytes(&key(Key::Tab, CTRL_SHIFT), CTRL_SHIFT),
        Input::Leave { backward: true }
    );
}

#[test]
fn alt_and_a_letter_sends_escape_first_and_skips_the_text() {
    assert_eq!(
        bytes(&key(Key::B, Modifiers::ALT), Modifiers::ALT),
        Input::Bytes(b"\x1bb".to_vec())
    );
    assert_eq!(
        bytes(&Event::Text("b".to_owned()), Modifiers::ALT),
        Input::Nothing
    );
}

#[test]
fn released_keys_send_nothing() {
    let released = Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    };

    assert_eq!(bytes(&released, Modifiers::NONE), Input::Nothing);
}
