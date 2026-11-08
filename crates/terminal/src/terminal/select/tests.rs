use crate::testing::fed;
use crate::{CellPoint, Terminal};

const fn at(line: usize, column: usize) -> CellPoint {
    CellPoint { line, column }
}

fn selected(terminal: &Terminal, (start, end): (CellPoint, CellPoint)) -> String {
    terminal.selection_text(start, end)
}

#[test]
fn a_word_ends_at_spaces() {
    let terminal = fed(40, 4, b"sudo pacman -Syu now");

    let word = terminal.word_bounds(at(0, 7));

    assert_eq!(word, (at(0, 5), at(0, 10)));
    assert_eq!(selected(&terminal, word), "pacman");
}

#[test]
fn a_word_ends_at_brackets_and_quotes() {
    let terminal = fed(40, 4, b"error: 'linux-zen' failed");

    assert_eq!(
        selected(&terminal, terminal.word_bounds(at(0, 10))),
        "linux-zen"
    );
}

#[test]
fn a_line_is_the_whole_row() {
    let terminal = fed(40, 4, b"first\r\nsecond row here\r\nthird");

    let line = terminal.line_bounds(at(1, 3));

    assert_eq!(line.0, at(1, 0));
    assert_eq!(selected(&terminal, line).trim_end(), "second row here");
}

#[test]
fn a_wrapped_line_is_selected_as_one() {
    let terminal = fed(10, 4, b"0123456789abcdef\r\nnext");

    let line = terminal.line_bounds(at(1, 2));

    assert_eq!(line.0, at(0, 0));
    assert_eq!(selected(&terminal, line).trim_end(), "0123456789abcdef");
}
