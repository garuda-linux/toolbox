//! Program output as lines of styled runs, for a view that draws the
//! colours itself. Only the text colour and bold survive:
//! titles, hyperlinks, cursor movement and stray control characters go.

#[cfg(test)]
mod tests;

use std::str::Split;

use crate::ansi::{ESCAPE, read_escape};
use crate::style::{Color, Run, Style};

const TAB_STOP: usize = 8;

fn apply(style: &mut Style, params: &str) {
    let mut codes = params.split(';');
    while let Some(code) = codes.next() {
        // systemd writes 256 colours with sub-parameters: `38:5:185`.
        if let Some((kind, rest)) = code.split_once(':') {
            if kind == "38" {
                style.fg = extended(&mut rest.split(':'));
            }

            continue;
        }

        // `ESC[m` is a reset, so an empty code counts as 0.
        let number = if code.is_empty() {
            Some(0)
        } else {
            code.parse::<u8>().ok()
        };

        match number {
            Some(0) => *style = Style::default(),
            Some(1) => style.bold = true,
            Some(22) => style.bold = false,
            Some(39) => style.fg = Color::Foreground,
            Some(basic @ 30..=37) => style.fg = Color::Indexed(basic - 30),
            Some(bright @ 90..=97) => style.fg = Color::Indexed(bright - 90 + 8),
            Some(38) => style.fg = extended(&mut codes),
            // The view draws no background or underline colour, but their
            // parameters must not be read as codes of their own.
            Some(48 | 58) => {
                extended(&mut codes);
            }
            _ => {}
        }
    }
}

/// The lines of `text`, each as its runs. A style carries over line ends,
/// as it does in a terminal. A last line break adds no empty line.
#[must_use = "the lines are returned, the input is unchanged"]
pub fn styled_lines(text: &str) -> Vec<Vec<Run>> {
    let mut lines = Vec::new();
    let mut line: Vec<Run> = Vec::new();
    let mut column = 0;
    let mut style = Style::default();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ESCAPE => {
                let mut sequence = String::new();
                read_escape(&mut chars, Some(&mut sequence));
                if let Some(params) = sequence
                    .strip_prefix('[')
                    .and_then(|body| body.strip_suffix('m'))
                {
                    apply(&mut style, params);
                }
            }
            '\n' => {
                lines.push(std::mem::take(&mut line));
                column = 0;
            }
            '\t' => {
                let spaces = TAB_STOP - column % TAB_STOP;
                push(&mut line, style, &" ".repeat(spaces), spaces);
                column += spaces;
            }
            other if other.is_control() => {}
            other => {
                push(&mut line, style, other.encode_utf8(&mut [0; 4]), 1);
                column += 1;
            }
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    lines
}

fn push(line: &mut Vec<Run>, style: Style, text: &str, columns: usize) {
    if let Some(last) = line.last_mut()
        && last.style == style
    {
        last.text.push_str(text);
        last.columns += columns;
        return;
    }

    line.push(Run {
        text: text.to_owned(),
        columns,
        style,
    });
}

/// The colour after `38`: a palette index after `5`, or red, green and blue
/// after `2`. The theme's text colour for anything else. An empty
/// sub-parameter (the colour space of `38:2::r:g:b`) is skipped.
fn extended(codes: &mut Split<'_, char>) -> Color {
    let mut number = || {
        codes
            .by_ref()
            .find(|code| !code.is_empty())
            .and_then(|code| code.parse::<u8>().ok())
    };

    let asked = match number() {
        Some(5) => number().map(Color::Indexed),
        Some(2) => number().and_then(|red| {
            let green = number()?;
            Some(Color::Rgb(red, green, number()?))
        }),
        _ => None,
    };

    asked.unwrap_or(Color::Foreground)
}
