use crate::testing::{numbered_lines, texts};
use crate::{Screen, Terminal};

const ROWS: u16 = 4;

fn ten_lines() -> Terminal {
    crate::testing::fed(10, ROWS, &numbered_lines(0..10))
}

fn first_text(screen: &Screen) -> String {
    screen.row_texts().swap_remove(0)
}

#[test]
fn the_live_view_shows_the_screen() {
    let terminal = ten_lines();

    assert_eq!(terminal.display_offset(), 0);
    assert_eq!(texts(&terminal), ["line7", "line8", "line9", ""]);
}

#[test]
fn scrolling_up_shows_older_lines() {
    let mut terminal = ten_lines();

    terminal.scroll(3);

    assert_eq!(terminal.display_offset(), 3);
    assert_eq!(first_text(&terminal.view()), "line4");
}

#[test]
fn scrolling_stops_at_the_oldest_and_newest_line() {
    let mut terminal = ten_lines();

    terminal.scroll(100);
    assert_eq!(terminal.display_offset(), terminal.history_len());
    assert_eq!(first_text(&terminal.view()), "line0");

    terminal.scroll(-1000);
    assert_eq!(terminal.display_offset(), 0);
}

#[test]
fn a_scrolled_view_stays_on_its_lines_while_output_arrives() {
    let mut terminal = ten_lines();
    terminal.scroll(3);

    terminal.feed(&numbered_lines(10..15));

    assert_eq!(first_text(&terminal.view()), "line4");
}

#[test]
fn scrolling_to_the_bottom_returns_to_the_live_screen() {
    let mut terminal = ten_lines();
    terminal.scroll(3);

    terminal.scroll_to_bottom();

    assert_eq!(terminal.display_offset(), 0);
}

#[test]
fn the_cursor_moves_down_with_the_view_and_hides_below_it() {
    let mut terminal = ten_lines();
    let live = terminal.view().cursor;

    terminal.scroll(1);
    let one_up = terminal.view().cursor;
    assert_eq!(one_up.row, live.row + 1);
    assert_eq!(one_up.visible, live.row + 1 < usize::from(ROWS));

    terminal.scroll(3);
    assert!(!terminal.view().cursor.visible);
}
