use alacritty_terminal::grid::Dimensions as _;
use alacritty_terminal::term::TermMode;

use super::{Dims, SCROLLBACK_LINES};
use crate::testing::{fed, numbered_lines, texts};
use crate::{CellPoint, Terminal, TerminalSize};

const fn size(columns: u16, rows: u16) -> TerminalSize {
    TerminalSize { columns, rows }
}

fn on_alternate_screen(terminal: &Terminal) -> bool {
    terminal.term.mode().contains(TermMode::ALT_SCREEN)
}

#[test]
fn the_alternate_screen_comes_and_goes() {
    let mut terminal = fed(10, 3, b"main\r\nshell");
    assert!(!on_alternate_screen(&terminal));

    terminal.feed(b"\x1b[?1049h\x1b[H\x1b[2Jeditor");
    assert!(on_alternate_screen(&terminal));
    assert_eq!(texts(&terminal), ["editor", "", ""]);

    terminal.feed(b"\x1b[?1049l");
    assert!(!on_alternate_screen(&terminal));
    assert_eq!(texts(&terminal), ["main", "shell", ""]);
    assert_eq!(terminal.view().cursor.row, 1);
}

#[test]
fn lines_scrolled_off_the_top_stay_in_the_scrollback() {
    let mut terminal = fed(10, 3, &numbered_lines(0..10));
    let mut scrolled = |lines: i32| {
        terminal.scroll(lines);
        terminal.view().row_texts()
    };

    assert_eq!(scrolled(0_i32), ["line8", "line9", ""]);
    assert_eq!(scrolled(1_i32), ["line7", "line8", "line9"]);
    assert_eq!(scrolled(7_i32), ["line0", "line1", "line2"]);
    assert_eq!(scrolled(500_i32), ["line0", "line1", "line2"]);
    assert_eq!(terminal.history_len(), 8);
}

#[test]
fn the_scrollback_is_bounded() {
    let terminal = fed(10, 3, &numbered_lines(0..SCROLLBACK_LINES + 50));

    assert_eq!(terminal.history_len(), SCROLLBACK_LINES);
}

#[test]
fn plain_text_covers_scrollback_and_screen() {
    let terminal = fed(10, 3, &numbered_lines(0..5));

    assert_eq!(
        terminal.to_plain_text(),
        "line0\nline1\nline2\nline3\nline4"
    );
}

#[test]
fn plain_text_joins_wrapped_lines() {
    let terminal = fed(4, 3, b"abcdefg\r\nh");

    assert_eq!(terminal.to_plain_text(), "abcdefg\nh");
}

#[test]
fn selection_reads_cells_between_two_points() {
    let terminal = fed(20, 3, b"hello world\r\nsecond line");

    let history = terminal.history_len();
    let word = terminal.selection_text(
        CellPoint {
            line: history,
            column: 6,
        },
        CellPoint {
            line: history,
            column: 10,
        },
    );

    assert_eq!(word, "world");

    let across = terminal.selection_text(
        CellPoint {
            line: history + 1,
            column: 5,
        },
        CellPoint {
            line: history,
            column: 6,
        },
    );

    assert_eq!(across, "world\nsecond");
}

#[test]
fn selection_clamps_points_outside_the_content() {
    let terminal = fed(5, 2, b"ab");

    let text = terminal.selection_text(
        CellPoint::default(),
        CellPoint {
            line: 99,
            column: 99,
        },
    );

    assert_eq!(text.trim_end(), "ab");
}

#[test]
fn resize_keeps_the_content() {
    let mut terminal = fed(10, 3, b"one\r\ntwo");

    terminal.resize(size(20, 5));
    assert_eq!(terminal.size(), size(20, 5));
    assert_eq!(terminal.view().rows.len(), 5);
    assert_eq!(
        terminal
            .view()
            .rows
            .first()
            .map(|row| row.iter().map(|run| run.columns).sum()),
        Some(20_usize)
    );
    assert_eq!(texts(&terminal).first().map(String::as_str), Some("one"));
}

#[test]
fn a_degenerate_size_still_works() {
    let terminal = fed(0, 0, b"abc");

    assert_eq!(terminal.view().rows.len(), 1);
}

#[test]
fn status_and_attribute_queries_are_answered() {
    let mut terminal = fed(10, 3, b"ab\x1b[6n");
    assert_eq!(terminal.take_replies(), b"\x1b[1;3R");
    assert_eq!(terminal.take_replies(), b"");

    terminal.feed(b"\x1b[c\x1b[5n");
    assert_eq!(terminal.take_replies(), b"\x1b[?6c\x1b[0n");
}

#[test]
fn key_mode_follows_the_program() {
    let mut terminal = Terminal::new(size(10, 3));
    assert!(!terminal.key_mode().application_cursor);
    assert!(!terminal.key_mode().bracketed_paste);

    terminal.feed(b"\x1b[?1h\x1b[?2004h");
    assert!(terminal.key_mode().application_cursor);
    assert!(terminal.key_mode().bracketed_paste);

    terminal.feed(b"\x1b[?1l");
    assert!(!terminal.key_mode().application_cursor);
}

#[test]
fn sequences_split_across_feeds_are_joined() {
    let mut terminal = Terminal::new(size(10, 2));
    terminal.feed(b"\x1b[3");
    terminal.feed(b"1mr");
    terminal.feed(&"日".as_bytes()[..1]);
    terminal.feed(&"日".as_bytes()[1..]);

    assert_eq!(texts(&terminal).first().map(String::as_str), Some("r日"));
}

#[test]
fn synchronized_updates_apply_at_once() {
    let terminal = fed(10, 2, b"\x1b[?2026hdrawn");

    assert_eq!(texts(&terminal).first().map(String::as_str), Some("drawn"));
}

#[test]
fn text_fed_as_str_matches_bytes() {
    let mut terminal = Terminal::new(size(10, 2));
    terminal.feed_str("\x1b[32mok\x1b[0m");

    assert_eq!(texts(&terminal).first().map(String::as_str), Some("ok"));
}

#[test]
fn the_terminal_can_move_into_a_task() {
    fn assert_send<T>()
    where
        T: Send,
    {
    }

    assert_send::<Terminal>();
}

#[test]
fn the_engine_size_reports_lines_and_columns() {
    let dims = Dims::of(size(0, 0));

    assert_eq!(dims.total_lines(), 1);
    assert_eq!(dims.screen_lines(), 1);
    assert_eq!(dims.columns(), 2);
}
