use super::styled_lines;
use crate::{Color, Run, Style};

fn run(text: &str, fg: Color, bold: bool) -> Run {
    Run {
        text: text.to_owned(),
        columns: text.chars().count(),
        style: Style {
            fg,
            bold,
            ..Style::default()
        },
    }
}

#[test]
fn plain_text_is_one_run_per_line() {
    assert_eq!(
        styled_lines("one\r\ntwo\n"),
        [
            vec![run("one", Color::Foreground, false)],
            vec![run("two", Color::Foreground, false)]
        ]
    );
}

#[test]
fn colours_and_bold_start_and_end_runs() {
    let lines = styled_lines("a \u{1b}[1;31merror\u{1b}[0m b");

    assert_eq!(
        lines,
        [vec![
            run("a ", Color::Foreground, false),
            run("error", Color::Indexed(1), true),
            run(" b", Color::Foreground, false),
        ]]
    );
}

#[test]
fn a_colour_carries_over_to_the_next_line() {
    let lines = styled_lines("\u{1b}[33mfirst\nsecond\u{1b}[39m");

    assert_eq!(
        lines,
        [
            vec![run("first", Color::Indexed(3), false)],
            vec![run("second", Color::Indexed(3), false)],
        ]
    );
}

#[test]
fn bright_256_and_true_colours_are_kept() {
    let lines = styled_lines("\u{1b}[92ma\u{1b}[38;5;185mb\u{1b}[38:5:9mc\u{1b}[38;2;1;2;3md");

    assert_eq!(
        lines,
        [vec![
            run("a", Color::Indexed(10), false),
            run("b", Color::Indexed(185), false),
            run("c", Color::Indexed(9), false),
            run("d", Color::Rgb(1, 2, 3), false),
        ]]
    );
}

#[test]
fn other_escapes_and_control_characters_are_dropped() {
    let lines = styled_lines("\u{1b}]0;title\u{7}x\u{1b}[2Ky\u{8}z");

    assert_eq!(lines, [vec![run("xyz", Color::Foreground, false)]]);
}

#[test]
fn tabs_reach_the_next_stop_of_eight() {
    assert_eq!(
        styled_lines("ab\tc"),
        [vec![run("ab      c", Color::Foreground, false)]]
    );
}

#[test]
fn an_empty_text_has_no_lines() {
    assert_eq!(styled_lines(""), Vec::<Vec<Run>>::new());
}
