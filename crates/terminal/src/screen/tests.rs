use crate::testing::{fed, texts};
use crate::{Color, CursorShape, Run, Style, Terminal};

const PLAIN: Style = Style {
    fg: Color::Foreground,
    bg: Color::Background,
    bold: false,
    italic: false,
    underline: false,
    dim: false,
    strikeout: false,
};

fn first_runs(terminal: &Terminal) -> Vec<Run> {
    terminal.view().rows.into_iter().next().unwrap_or_default()
}

fn style_of(terminal: &Terminal, text: &str) -> Style {
    first_runs(terminal)
        .into_iter()
        .find(|run| run.text == text)
        .map(|run| run.style)
        .unwrap()
}

#[test]
fn plain_text_fills_one_run_across_the_row() {
    let terminal = fed(10, 2, b"hello");

    assert_eq!(
        first_runs(&terminal),
        [Run {
            text: "hello     ".to_owned(),
            columns: 10,
            style: PLAIN
        }]
    );
    assert_eq!(terminal.view().rows.len(), 2);
}

#[test]
fn sgr_colours_become_theme_friendly_colours() {
    let terminal = fed(
        60,
        2,
        b"\x1b[31mred\x1b[0m\x1b[91mbri\x1b[0m\x1b[38;5;208mpal\x1b[0m\x1b[38;2;1;2;3mrgb\x1b[0m\x1b[44mbg4\x1b[0m\x1b[48;5;100mbgp\x1b[0m",
    );

    assert_eq!(style_of(&terminal, "red").fg, Color::Indexed(1));
    assert_eq!(style_of(&terminal, "bri").fg, Color::Indexed(9));
    assert_eq!(style_of(&terminal, "pal").fg, Color::Indexed(208));
    assert_eq!(style_of(&terminal, "rgb").fg, Color::Rgb(1, 2, 3));
    assert_eq!(style_of(&terminal, "bg4").bg, Color::Indexed(4));
    assert_eq!(style_of(&terminal, "bgp").bg, Color::Indexed(100));
    assert_eq!(style_of(&terminal, "bgp").fg, Color::Foreground);
}

#[test]
fn bold_italic_underline_dim_and_strikeout() {
    let terminal = fed(
        40,
        1,
        b"\x1b[1mbo\x1b[0m\x1b[3mit\x1b[0m\x1b[4mun\x1b[0m\x1b[2mdi\x1b[0m\x1b[9mst\x1b[0m",
    );

    assert!(style_of(&terminal, "bo").bold);
    assert!(style_of(&terminal, "it").italic);
    assert!(style_of(&terminal, "un").underline);
    assert!(style_of(&terminal, "di").dim);
    assert!(style_of(&terminal, "st").strikeout);
    assert!(!style_of(&terminal, "st").bold);
}

#[test]
fn inverse_swaps_the_colours_before_the_ui_sees_them() {
    let terminal = fed(20, 1, b"\x1b[7minv\x1b[0m\x1b[31;7mred\x1b[0m");

    let inverse = style_of(&terminal, "inv");
    assert_eq!(
        (inverse.fg, inverse.bg),
        (Color::Background, Color::Foreground)
    );

    let red = style_of(&terminal, "red");
    assert_eq!((red.fg, red.bg), (Color::Background, Color::Indexed(1)));
}

#[test]
fn equal_neighbours_merge_and_style_changes_split() {
    let terminal = fed(6, 1, b"a\x1b[1mb\x1b[1mc\x1b[0md");

    let runs: Vec<(String, usize)> = first_runs(&terminal)
        .into_iter()
        .map(|run| (run.text, run.columns))
        .collect();

    assert_eq!(
        runs,
        [
            ("a".to_owned(), 1),
            ("bc".to_owned(), 2),
            ("d  ".to_owned(), 3)
        ]
    );
}

#[test]
fn carriage_return_overwrites_like_a_progress_bar() {
    let terminal = fed(20, 2, b"( 1/2) 10%\r( 1/2) 100%");

    assert_eq!(texts(&terminal), ["( 1/2) 100%", ""]);
}

#[test]
fn clear_screen_leaves_only_what_follows() {
    let terminal = fed(10, 3, b"junk\r\nmore\x1b[2J\x1b[Hfresh");

    assert_eq!(texts(&terminal), ["fresh", "", ""]);
}

#[test]
fn cursor_addressing_places_text_and_cursor() {
    let terminal = fed(20, 6, b"\x1b[5;10HX");

    let rows = texts(&terminal);
    assert_eq!(rows.get(4).map(String::as_str), Some("         X"));

    let cursor = terminal.view().cursor;
    assert_eq!((cursor.row, cursor.column), (4, 10));
    assert!(cursor.visible);
    assert_eq!(cursor.shape, CursorShape::Block);
}

#[test]
fn programs_hide_and_reshape_the_cursor() {
    assert!(!fed(10, 2, b"\x1b[?25l").view().cursor.visible);
    assert_eq!(
        fed(10, 2, b"\x1b[5 q").view().cursor.shape,
        CursorShape::Beam
    );
    assert_eq!(
        fed(10, 2, b"\x1b[3 q").view().cursor.shape,
        CursorShape::Underline
    );
}

#[test]
fn wide_characters_take_two_cells() {
    let terminal = fed(10, 1, "日本x".as_bytes());

    let runs = first_runs(&terminal);
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs.first().map(|run| run.text.as_str()),
        Some("日本x     ")
    );
    assert_eq!(runs.first().map(|run| run.columns), Some(10));
    assert_eq!(terminal.view().cursor.column, 5);
}

#[test]
fn a_wide_character_that_does_not_fit_wraps_whole() {
    let terminal = fed(3, 2, "ab日".as_bytes());

    assert_eq!(texts(&terminal), ["ab", "日"]);
    let widths: Vec<usize> = terminal
        .view()
        .rows
        .iter()
        .map(|row| row.iter().map(|run| run.columns).sum())
        .collect();

    assert_eq!(widths, [3, 3]);
}

#[test]
fn combining_marks_stay_with_their_character() {
    let terminal = fed(5, 1, "e\u{301}x".as_bytes());

    assert_eq!(texts(&terminal), ["e\u{301}x"]);
    assert_eq!(terminal.view().cursor.column, 2);
}

#[test]
fn hidden_text_keeps_its_cells_blank() {
    let terminal = fed(10, 1, b"\x1b[8msecret\x1b[0m!");

    assert_eq!(texts(&terminal), ["      !"]);
}
