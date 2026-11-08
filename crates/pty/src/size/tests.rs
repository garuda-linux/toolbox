use super::{DEFAULT_SIZE, clamped, winsize};
use crate::TerminalSize;

const fn size(columns: u16, rows: u16) -> TerminalSize {
    TerminalSize { columns, rows }
}

#[test]
fn sizes_are_clamped_to_what_programs_can_draw() {
    for (wanted, expected) in [
        (size(120, 30), size(120, 30)),
        (size(10, 2), size(40, 10)),
        (size(1000, 1000), size(400, 200)),
        (size(u16::MAX, u16::MAX), size(400, 200)),
        (size(0, 0), size(40, 10)),
    ] {
        assert_eq!(clamped(wanted), expected, "{wanted:?}");
    }
}

#[test]
fn the_default_size_needs_no_clamping() {
    assert_eq!(clamped(DEFAULT_SIZE), DEFAULT_SIZE);
}

#[test]
fn a_size_becomes_a_window_size_without_pixels() {
    let window = winsize(size(100, 25));
    assert_eq!(
        (
            window.ws_col,
            window.ws_row,
            window.ws_xpixel,
            window.ws_ypixel
        ),
        (100, 25, 0, 0)
    );
}
