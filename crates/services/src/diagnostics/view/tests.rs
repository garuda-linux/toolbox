use toolbox_pty::styled_lines;

use super::{Row, piece, rows};

const REPORT: &str =
    "kernel: hello\n\u{1b}[31mkernel: OOPS\u{1b}[0m at boot\nusb 1-1: new device\n";

fn row(line: usize, start: usize, end: usize) -> Row {
    Row { line, start, end }
}

#[test]
fn without_a_search_every_line_is_one_row() {
    let lines = styled_lines(REPORT);

    assert_eq!(
        rows(&lines, "  ", None),
        [row(0, 0, 13), row(1, 0, 20), row(2, 0, 19)]
    );
}

#[test]
fn a_search_keeps_the_lines_that_hold_it_in_any_case() {
    let lines = styled_lines(REPORT);

    assert_eq!(rows(&lines, "oops", None), [row(1, 0, 20)]);
    assert_eq!(rows(&lines, "nothing", None), []);
}

#[test]
fn wrapped_lines_are_cut_into_pieces_of_the_width() {
    let lines = styled_lines(REPORT);

    assert_eq!(
        rows(&lines, "", Some(8)),
        [
            row(0, 0, 8),
            row(0, 8, 13),
            row(1, 0, 8),
            row(1, 8, 16),
            row(1, 16, 20),
            row(2, 0, 8),
            row(2, 8, 16),
            row(2, 16, 19),
        ]
    );
}

#[test]
fn a_piece_keeps_the_colour_of_each_run_it_crosses() {
    let lines = styled_lines(REPORT);
    let second = &lines[1];
    let parts = piece(second, 4, 16);
    let texts: Vec<&str> = parts.iter().map(|(text, _)| text.as_str()).collect();

    assert_eq!(texts, ["el: OOPS", " at "]);
    assert_ne!(parts[0].1.style, parts[1].1.style);
}
