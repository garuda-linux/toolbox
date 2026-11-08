//! How answers are printed: a bold title with a summary, rows marked by
//! their state, later columns dimmed, `hint:` lines. Colour and marks only
//! on a terminal that did not ask for none, JSON for scripts.

mod json;
mod row;
mod size;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt::Write as _;

use anstyle::{AnsiColor, Color};
use unicode_width::{UnicodeWidthChar as _, UnicodeWidthStr as _};

pub use self::json::{SCHEMA, json, json_line, with_schema};
pub use self::row::{Mark, Row};
pub use self::size::size;

const GAP: usize = 2;

/// A cut column keeps at least this many columns. Below it a wrapped line
/// reads better than a cut one.
const NARROWEST: usize = 10;

/// Lines under a title start this far in.
pub const INDENT: &str = "  ";

const BOLD: anstyle::Style = anstyle::Style::new().bold();
const DIM: anstyle::Style = anstyle::Style::new().dimmed();
const BLUE: anstyle::Style = BOLD.fg_color(Some(Color::Ansi(AnsiColor::Blue)));
const RED: anstyle::Style = BOLD.fg_color(Some(Color::Ansi(AnsiColor::Red)));
const YELLOW: anstyle::Style = BOLD.fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
const CYAN: anstyle::Style = BOLD.fg_color(Some(Color::Ansi(AnsiColor::Cyan)));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    Color,
}

/// `--color`: what the user asked for, before the environment and the
/// terminal have their say.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

/// The command line's look. [`Style::Plain`] paints nothing and leaves
/// the marks out.
#[derive(Clone, Copy, Debug)]
pub struct Paint {
    style: Style,
    width: Option<usize>,
    quiet: bool,
}

impl Paint {
    pub const fn new(style: Style) -> Self {
        Self {
            style,
            width: None,
            quiet: false,
        }
    }

    /// The terminal's width in columns. A list wider than it cuts its
    /// widest column after the name, `None` cuts nothing.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn width(self, width: Option<usize>) -> Self {
        Self { width, ..self }
    }

    /// `--quiet`: the hints that say what to do next are left out.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn quiet(self, quiet: bool) -> Self {
        Self { quiet, ..self }
    }

    pub const fn style(self) -> Style {
        self.style
    }

    pub const fn is_quiet(self) -> bool {
        self.quiet
    }

    fn painted(self, look: anstyle::Style, text: &str) -> String {
        match self.style {
            Style::Plain => text.to_owned(),
            Style::Color => format!("{}{text}{}", look.render(), look.render_reset()),
        }
    }

    /// `title` in bold, then `summary` dimmed in brackets.
    pub fn title(self, title: &str, summary: Option<&str>) -> String {
        let title = self.painted(BOLD, title);
        match summary {
            Some(summary) => format!("{title}  {}", self.painted(DIM, &format!("({summary})"))),
            None => title,
        }
    }

    pub fn item(self, text: &str) -> String {
        format!("{INDENT}{text}")
    }

    pub fn error(self, text: &str) -> String {
        format!("{} {text}", self.painted(RED, "error:"))
    }

    pub fn warning(self, text: &str) -> String {
        format!("{} {text}", self.painted(YELLOW, "warning:"))
    }

    /// What to do next, nothing with `--quiet`.
    pub fn hint(self, text: &str) -> String {
        if self.quiet {
            return String::new();
        }

        format!("{} {text}", self.painted(CYAN, "hint:"))
    }

    /// A program's own words under the line they explain, dimmed.
    pub fn detail(self, text: &str) -> String {
        text.lines().fold(String::new(), |mut lines, line| {
            writeln!(lines, "    {}", self.painted(DIM, line)).ok();
            lines
        })
    }

    /// `text` that matters less than the words around it.
    pub fn dim(self, text: &str) -> String {
        self.painted(DIM, text)
    }

    pub fn question(self, text: &str) -> String {
        self.painted(BOLD, text)
    }

    /// A titled list: an optional header row, then the rows in columns as
    /// wide as their widest cell. On a terminal each row starts with its
    /// mark and the columns after the name are dimmed.
    pub fn list(self, title: &str, summary: Option<&str>, head: &[&str], rows: &[Row]) -> String {
        let mut text = format!("{}\n\n", self.title(title, summary));
        if rows.is_empty() {
            writeln!(text, "{}", self.item(&self.painted(DIM, "none"))).ok();
            return text;
        }

        let head: Vec<String> = head.iter().map(|cell| (*cell).to_owned()).collect();
        let mut widths = widths(rows.iter().map(|row| &row.cells).chain([&head]));
        let marked = self.style == Style::Color && rows.iter().any(|row| row.mark != Mark::Blank);
        let room = if marked { "  " } else { "" };
        if let Some(width) = self.width {
            fit(&mut widths, width.saturating_sub(INDENT.len() + room.len()));
        }

        if !head.is_empty() {
            let line = line(&head, &widths, |cell, _| cell.to_owned());
            writeln!(text, "{INDENT}{room}{}", self.painted(DIM, &line)).ok();
        }

        for row in rows {
            let sign = match (marked, row.mark.sign()) {
                (true, Some((sign, look))) => format!("{} ", self.painted(look, sign)),
                (true, None) => room.to_owned(),
                (false, _) => String::new(),
            };

            let cells = line(&row.cells, &widths, |cell, column| {
                if column == 0 {
                    cell.to_owned()
                } else {
                    self.painted(DIM, cell)
                }
            });

            writeln!(text, "{INDENT}{sign}{cells}").ok();
        }

        text
    }

    /// A titled section of `name  value` lines, the values lined up.
    pub fn facts(self, title: &str, facts: &[(&str, &str)]) -> String {
        let width = facts
            .iter()
            .map(|(name, _)| name.chars().count())
            .max()
            .unwrap_or(0);

        let mut text = format!("{}\n\n", self.title(title, None));
        for (name, value) in facts {
            let value = toolbox_plan::text::plain(value);
            let room = width - name.chars().count() + GAP;
            let line = format!("{}{:room$}{value}", self.painted(DIM, name), "");
            writeln!(text, "{}", self.item(line.trim_end())).ok();
        }

        text
    }

    /// clap's help and errors in the same look: headers blue and bold,
    /// literals bold, errors red. clap itself leaves colour out without a
    /// terminal or with `NO_COLOR`.
    pub const fn help() -> clap::builder::Styles {
        clap::builder::Styles::styled()
            .header(BLUE)
            .usage(BLUE)
            .literal(BOLD)
            .placeholder(anstyle::Style::new())
            .error(RED)
            .valid(BOLD)
            .invalid(YELLOW)
    }
}

/// What the environment says about colour, for `--color auto`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorEnv {
    /// `NO_COLOR` with a value (no-color.org), `CLICOLOR=0` or
    /// `TERM=dumb`: a terminal stays plain.
    pub refused: bool,
    /// `CLICOLOR_FORCE` or `FORCE_COLOR` other than `0`: even a pipe is
    /// coloured, and this wins over a refusal.
    pub forced: bool,
}

impl ColorEnv {
    /// The variables as `lookup` finds them, usually `std::env::var_os`.
    pub fn read(lookup: impl Fn(&str) -> Option<OsString>) -> Self {
        let is = |key: &str, wanted: &str| lookup(key).is_some_and(|value| value == wanted);
        let set = |key: &str| lookup(key).is_some_and(|value| !value.is_empty());

        Self {
            refused: set("NO_COLOR") || is("CLICOLOR", "0") || is("TERM", "dumb"),
            forced: ["CLICOLOR_FORCE", "FORCE_COLOR"]
                .into_iter()
                .any(|key| set(key) && !is(key, "0")),
        }
    }
}

/// The look asked for. `--color` decides first: `always` colours even a
/// pipe, `never` leaves even a terminal plain. `auto`, the default,
/// colours a terminal unless `env` refuses, and a pipe when `env` forces.
pub const fn style(mode: ColorMode, env: ColorEnv, terminal: bool) -> Style {
    let coloured = match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => env.forced || (terminal && !env.refused),
    };

    if coloured { Style::Color } else { Style::Plain }
}

fn widths<'a>(rows: impl Iterator<Item = &'a Vec<String>>) -> Vec<usize> {
    rows.fold(Vec::new(), |mut widths, row| {
        for (column, cell) in row.iter().enumerate() {
            let width = cell.width();
            match widths.get_mut(column) {
                Some(known) => *known = width.max(*known),
                None => widths.push(width),
            }
        }

        widths
    })
}

/// `cells` padded to `widths` and cut to them, each painted by `look`,
/// without the empty cells at the end.
fn line(cells: &[String], widths: &[usize], look: impl Fn(&str, usize) -> String) -> String {
    let used = cells
        .iter()
        .rposition(|cell| !cell.is_empty())
        .map_or(0, |last| last + 1);

    let mut line = String::new();

    for (column, (cell, width)) in cells.iter().zip(widths).take(used).enumerate() {
        let cell = cut(cell, *width);
        line.push_str(&look(&cell, column));

        if column + 1 < used {
            let room = width.saturating_sub(cell.width()) + GAP;
            write!(line, "{:room$}", "").ok();
        }
    }

    line
}

/// Narrows the widest column after the name so a row fits in `room`. The
/// last of equally wide ones goes, since descriptions and paths come last.
fn fit(widths: &mut [usize], room: usize) {
    let used = widths.iter().sum::<usize>() + GAP * widths.len().saturating_sub(1);
    let over = used.saturating_sub(room);
    if over == 0 {
        return;
    }

    let Some(widest) = widths.iter_mut().skip(1).max_by_key(|width| **width) else {
        return;
    };

    if let Some(left) = widest.checked_sub(over).filter(|left| *left >= NARROWEST) {
        *widest = left;
    }
}

/// `cell` within `width` display columns, ending in `…` when it was cut.
fn cut(cell: &str, width: usize) -> Cow<'_, str> {
    if cell.width() <= width {
        return Cow::Borrowed(cell);
    }

    let mut used = 0;
    let mut text = String::new();
    for char in cell.chars() {
        let wide = char.width().unwrap_or(0);
        if used + wide >= width {
            break;
        }

        used += wide;
        text.push(char);
    }

    text.push('…');
    Cow::Owned(text)
}
