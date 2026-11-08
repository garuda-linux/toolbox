use super::{
    MouseAction, MouseButton, MouseEncoding, MouseEvent, MouseMode, MouseReporting,
    encode_alternate_scroll, encode_mouse,
};
use crate::{Modifiers, Terminal, TerminalSize};

const CLICKS: MouseMode = MouseMode {
    reporting: MouseReporting::Clicks,
    encoding: MouseEncoding::Classic,
    alternate_scroll: false,
};
const SGR: MouseMode = MouseMode {
    encoding: MouseEncoding::Sgr,
    ..CLICKS
};

const fn event(button: Option<MouseButton>, action: MouseAction) -> MouseEvent {
    MouseEvent {
        button,
        action,
        column: 4,
        row: 1,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn nothing_is_sent_while_the_program_asked_for_no_mouse() {
    let off = MouseMode::default();

    assert_eq!(
        encode_mouse(event(Some(MouseButton::Left), MouseAction::Press), off),
        None
    );
}

#[test]
fn a_click_is_sent_in_the_classic_form() {
    let press = encode_mouse(event(Some(MouseButton::Left), MouseAction::Press), CLICKS);
    let release = encode_mouse(event(Some(MouseButton::Left), MouseAction::Release), CLICKS);

    assert_eq!(press, Some(b"\x1b[M\x20\x25\x22".to_vec()));
    assert_eq!(release, Some(b"\x1b[M\x23\x25\x22".to_vec()));
}

#[test]
fn sgr_names_the_button_on_release_too() {
    let press = encode_mouse(event(Some(MouseButton::Right), MouseAction::Press), SGR);
    let release = encode_mouse(event(Some(MouseButton::Right), MouseAction::Release), SGR);

    assert_eq!(press, Some(b"\x1b[<2;5;2M".to_vec()));
    assert_eq!(release, Some(b"\x1b[<2;5;2m".to_vec()));
}

#[test]
fn modifiers_and_the_wheel_change_the_button_code() {
    let mut wheel = event(Some(MouseButton::WheelUp), MouseAction::Press);
    wheel.modifiers = Modifiers::CONTROL;

    assert_eq!(encode_mouse(wheel, SGR), Some(b"\x1b[<80;5;2M".to_vec()));
}

#[test]
fn motion_is_sent_only_in_the_mode_that_asks_for_it() {
    let dragging = event(Some(MouseButton::Left), MouseAction::Motion);
    let hovering = event(None, MouseAction::Motion);
    let drag = MouseMode {
        reporting: MouseReporting::Drag,
        ..SGR
    };

    let motion = MouseMode {
        reporting: MouseReporting::Motion,
        ..SGR
    };

    assert_eq!(encode_mouse(dragging, SGR), None);
    assert_eq!(
        encode_mouse(dragging, drag),
        Some(b"\x1b[<32;5;2M".to_vec())
    );
    assert_eq!(encode_mouse(hovering, drag), None);
    assert_eq!(
        encode_mouse(hovering, motion),
        Some(b"\x1b[<35;5;2M".to_vec())
    );
}

#[test]
fn a_cell_beyond_the_classic_range_is_sent_only_in_utf8_mode() {
    let mut far = event(Some(MouseButton::Left), MouseAction::Press);
    far.column = 300;
    let utf8 = MouseMode {
        encoding: MouseEncoding::Utf8,
        ..CLICKS
    };

    assert_eq!(encode_mouse(far, CLICKS), None);
    let encoded = encode_mouse(far, utf8).unwrap();
    assert!(String::from_utf8(encoded).unwrap().contains('\u{14d}'));
}

#[test]
fn alternate_scroll_sends_arrows_per_line() {
    assert_eq!(encode_alternate_scroll(2), b"\x1bOA\x1bOA".to_vec());
    assert_eq!(encode_alternate_scroll(-1), b"\x1bOB".to_vec());
}

#[test]
fn the_terminal_reports_the_modes_the_program_set() {
    let mut terminal = Terminal::new(TerminalSize {
        columns: 20,
        rows: 4,
    });

    assert_eq!(terminal.mouse_mode(), MouseMode::default());

    terminal.feed(b"\x1b[?1000h\x1b[?1006h");
    let mode = terminal.mouse_mode();

    assert_eq!(mode.reporting, MouseReporting::Clicks);
    assert_eq!(mode.encoding, MouseEncoding::Sgr);
}
