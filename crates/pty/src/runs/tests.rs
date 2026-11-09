use super::styled_lines;
use crate::{Color, Run, Style};

const CONFLICT: &str = include_str!("../../tests/fixtures/pacman-conflict.txt");
const VERSION: &str = "4.0.0.r39705.g978212a-1";

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

fn style_of(lines: &[Vec<Run>], text: &str) -> Option<Style> {
    lines
        .iter()
        .flatten()
        .find(|run| run.text.contains(text))
        .map(|run| run.style)
}

fn bold() -> Style {
    Style {
        bold: true,
        ..Style::default()
    }
}

#[test]
fn a_pacman_transcript_keeps_its_256_colour() {
    let lines = styled_lines(CONFLICT);

    assert_eq!(
        style_of(&lines, VERSION),
        Some(Style {
            fg: Color::Indexed(243),
            ..bold()
        })
    );
}

#[test]
fn a_background_colour_leaves_the_text_style_alone() {
    // Changed: the version's 256 colour foreground becomes a background.
    let text = CONFLICT.replacen("38;5;243", "48;5;34", 1);

    assert_eq!(style_of(&styled_lines(&text), VERSION), Some(bold()));
}

#[test]
fn a_true_colour_background_leaves_the_text_style_alone() {
    // Changed: the version's 256 colour foreground becomes a true colour background.
    let text = CONFLICT.replacen("38;5;243", "48;2;31;1;22", 1);

    assert_eq!(style_of(&styled_lines(&text), VERSION), Some(bold()));
}

#[test]
fn an_underline_colour_leaves_the_text_style_alone() {
    // Changed: the version's 256 colour foreground becomes an underline colour.
    let text = CONFLICT.replacen("38;5;243", "58;5;22", 1);

    assert_eq!(style_of(&styled_lines(&text), VERSION), Some(bold()));
}

#[test]
fn an_out_of_range_code_is_ignored() {
    // Changed: the error prefix gains the out of range code 300.
    let text = CONFLICT.replacen("1;31merror", "1;300;31merror", 1);

    assert_eq!(
        style_of(&styled_lines(&text), "error: "),
        Some(Style {
            fg: Color::Indexed(1),
            ..bold()
        })
    );
}
