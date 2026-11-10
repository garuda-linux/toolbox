//! What every command prints the same way: lists, `error:` lines with
//! their hint and the colour choice. [`Out`] carries the choices of one
//! run and the errors it keeps, so nothing of a run is process wide.

#[cfg(test)]
mod tests;
mod text;

use std::io::{self, IsTerminal as _, Write};

use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::args::Read;
use toolbox_cli::output::{ColorEnv, ColorMode, Paint, Row, Style, json, style};

pub use self::text::{Said, error_object, failure_text};
use self::text::{Sink, failed_text, lost_sink, said, sink};

pub struct Listing<'a> {
    pub title: &'a str,
    /// The count and what stands out, in the title's brackets.
    pub summary: Option<String>,
    /// Column names, for lists whose columns do not explain themselves.
    pub head: &'a [&'a str],
    pub rows: Vec<Row>,
    /// What to do next, under the list.
    pub hint: Option<String>,
}

/// What `--json` asks of errors. A change's JSON Lines win over a read's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Json {
    #[default]
    Off,
    /// `--json` on a change: standard output holds JSON Lines only,
    /// errors included.
    Lines,
    /// `--json` on a read: each error is kept for its JSON object,
    /// written once the exit code is known.
    Read,
}

/// `--color`, `--width`, `--quiet`, `--verbose` and `--json`.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub color: ColorMode,
    pub width: Option<usize>,
    pub quiet: bool,
    pub verbose: bool,
    pub json: Json,
}

/// Where a command writes, with the choices it writes under.
pub struct Out<'w> {
    writer: &'w mut dyn Write,
    settings: Settings,
    /// The errors of a read under `--json`, in the order they came.
    kept: Vec<Said>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color: ColorMode::Auto,
            width: None,
            quiet: false,
            verbose: false,
            json: Json::Off,
        }
    }
}

impl<'w> Out<'w> {
    pub fn new(writer: &'w mut dyn Write, settings: Settings) -> Self {
        Self {
            writer,
            settings,
            kept: Vec::new(),
        }
    }

    pub const fn settings(&self) -> Settings {
        self.settings
    }

    pub const fn verbose(&self) -> bool {
        self.settings.verbose
    }

    pub const fn quiet(&self) -> bool {
        self.settings.quiet
    }

    /// Colour as `--color` and the environment ask.
    ///
    /// Never with `NO_COLOR`, `CLICOLOR=0` or `TERM=dumb` unless
    /// `--color=always` or a force variable. Long cells are cut to
    /// `--width`, `COLUMNS` or the terminal's width, a pipe gets them
    /// whole.
    pub fn paint(&self) -> Paint {
        paint(self.settings)
    }

    /// The width of standard output: `--width`, else `COLUMNS` or the
    /// terminal's. A pipe has none.
    pub fn columns(&self) -> Option<usize> {
        columns(self.settings)
    }

    /// The errors kept under `--json`, for [`kept_errors`] once the exit
    /// code is known.
    pub fn into_kept(self) -> Vec<Said> {
        self.kept
    }
}

impl Write for Out<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Rows as JSON for `--json`, else as the [`Listing`] `show` makes.
pub fn print<T>(
    read: Read,
    rows: &[T],
    out: &mut Out<'_>,
    show: impl FnOnce(&[T]) -> Listing<'_>,
) -> Exit
where
    T: Serialize,
{
    if read.json {
        write!(out, "{}", json(&rows)).ok();
        return Exit::Success;
    }

    write_listing(show(rows), out)
}

pub fn write_listing(listing: Listing<'_>, out: &mut Out<'_>) -> Exit {
    let paint = out.paint();
    let summary = listing.summary.as_deref();
    write!(
        out,
        "{}",
        paint.list(listing.title, summary, listing.head, &listing.rows)
    )
    .ok();

    if let Some(hint) = listing.hint
        && !paint.is_quiet()
    {
        writeln!(out, "\n{}", paint.hint(&hint)).ok();
    }

    Exit::Success
}

/// [`Out::paint`] for `settings`, for text made before a writer exists.
pub fn paint(settings: Settings) -> Paint {
    let terminal = io::stdout().is_terminal();
    Paint::new(style_on(settings.color, terminal))
        .width(columns(settings))
        .quiet(settings.quiet)
}

fn columns(settings: Settings) -> Option<usize> {
    let stdout = io::stdout();
    let terminal = stdout.is_terminal();
    settings
        .width
        .or_else(|| terminal.then(env_columns).flatten())
        .or_else(|| terminal.then(|| terminal_width(&stdout)).flatten())
}

/// The look on a stream that may be standard output or standard error.
pub fn style_on(color: ColorMode, terminal: bool) -> Style {
    style(color, ColorEnv::read(|key| std::env::var_os(key)), terminal)
}

fn env_columns() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()?
        .parse()
        .ok()
        .filter(|cols| *cols > 0)
}

fn terminal_width(stdout: &io::Stdout) -> Option<usize> {
    rustix::termios::tcgetwinsize(stdout)
        .ok()
        .map(|size| usize::from(size.ws_col))
        .filter(|cols| *cols > 0)
}

pub fn unavailable(what: &str, out: &mut Out<'_>) -> Exit {
    say_error(out, &format!("not available: {what}"));
    Exit::Unavailable
}

/// An `error:` line of the command's own, a second line of `text` its
/// hint. A read's `--json` keeps it for its object alone, so standard
/// output holds the JSON and standard error nothing but objects.
pub fn say_error(out: &mut Out<'_>, text: &str) {
    let settings = out.settings;
    match sink(settings.json) {
        Sink::Line => {
            write!(out, "{}", said(out.paint(), text, true)).ok();
        }
        Sink::Text => {
            write!(io::stderr(), "{}", said(out.paint(), text, false)).ok();
        }
        Sink::Kept => out.kept.push(Said::new(text, None)),
    }
}

/// `error: …` and its `hint: …` line, painted for standard error, as a
/// start problem prints it before any command runs.
pub fn error_text(text: &str) -> String {
    let paint = Paint::new(style_on(ColorMode::Auto, io::stderr().is_terminal()));
    said(paint, text, false)
}

/// `what` went wrong, as one line. A program's own words in `raw` follow
/// with `--verbose`, else a hint says how to see them. As with
/// [`say_error`], a read's `--json` keeps it for its object alone.
pub fn failure(out: &mut Out<'_>, what: &str, raw: &str) {
    let settings = out.settings;
    let text = failed_text(
        out.paint(),
        what,
        raw,
        settings.verbose,
        settings.json == Json::Lines,
    );

    match sink(settings.json) {
        Sink::Line => {
            write!(out, "{text}").ok();
        }
        Sink::Text => {
            write!(io::stderr(), "{text}").ok();
        }
        Sink::Kept => out.kept.push(Said::new(what, Some(raw))),
    }
}

/// The objects of the errors kept under `--json`, one line each, for
/// standard error. `exit` is the code the command ends with.
pub fn kept_errors(kept: &[Said], exit: Exit) -> String {
    kept.iter().map(|said| error_object(said, exit)).collect()
}

/// Output that never reached its reader, as one `error:` line on standard
/// error, or under any `--json` as an object for the end.
pub fn lost(settings: Settings, kind: io::ErrorKind) -> Option<Said> {
    let text = format!("the output was lost: {kind}");

    match lost_sink(settings.json) {
        Sink::Kept => Some(Said::new(&text, None)),
        Sink::Line | Sink::Text => {
            write!(io::stderr(), "{}", error_text(&text)).ok();
            None
        }
    }
}
