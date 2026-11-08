//! What every command prints the same way: lists, `error:` lines with
//! their hint and the colour choice.

#[cfg(test)]
mod tests;

use std::io::{IsTerminal as _, Write};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

use parking_lot::Mutex;
use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::args::Read;
use toolbox_cli::output::{ColorEnv, ColorMode, Paint, Row, Style, json, json_line, style};

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// `--json` on a change: standard output holds JSON Lines only, errors
/// included.
static JSON_LINES: AtomicBool = AtomicBool::new(false);

/// `--json` on a read: each error is kept for its JSON object, written once
/// the exit code is known.
static JSON_READ: AtomicBool = AtomicBool::new(false);

/// The errors of a read under `--json`, in the order they came.
static SAID: Mutex<Vec<Said>> = Mutex::new(Vec::new());

static QUIET: AtomicBool = AtomicBool::new(false);

/// `--color`: `0` is `auto`, `1` `always`, `2` `never`.
static COLOR: AtomicU8 = AtomicU8::new(0);

/// `--width`. `0` means the terminal's.
static WIDTH: AtomicUsize = AtomicUsize::new(0);

/// One error as a read's JSON gives it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Said {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
    /// The program's own words.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl Said {
    /// The error of `text`, a second line its hint, with the program's own
    /// words in `detail`.
    pub fn new(text: &str, detail: Option<&str>) -> Self {
        let (message, hint) = text
            .split_once('\n')
            .map_or((text, None), |(message, hint)| (message, Some(hint)));

        Self {
            message: message.to_owned(),
            hint: hint.map(str::to_owned),
            detail: detail
                .filter(|detail| !detail.trim().is_empty())
                .map(str::to_owned),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sink {
    /// A change's JSON Lines on standard output.
    Line,
    /// `error:` and `hint:` lines on standard error.
    Text,
    /// Kept for a read's JSON object on standard error.
    Kept,
}

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

/// Rows as JSON for `--json`, else as the [`Listing`] `show` makes.
pub fn print<T>(
    read: Read,
    rows: &[T],
    out: &mut dyn Write,
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

pub fn write_listing(listing: Listing<'_>, out: &mut dyn Write) -> Exit {
    let paint = paint();
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

/// Colour as `--color` and the environment ask.
///
/// Never with `NO_COLOR`, `CLICOLOR=0` or `TERM=dumb` unless
/// `--color=always` or a force variable. Long cells are cut to
/// `--width`, `COLUMNS` or the terminal's width, a pipe gets them whole.
pub fn paint() -> Paint {
    let terminal = std::io::stdout().is_terminal();
    Paint::new(style_on(terminal))
        .width(columns())
        .quiet(quiet())
}

/// The width of standard output: `--width`, else `COLUMNS` or the
/// terminal's. A pipe has none.
pub fn columns() -> Option<usize> {
    let stdout = std::io::stdout();
    let terminal = stdout.is_terminal();
    configured_width()
        .or_else(|| terminal.then(env_columns).flatten())
        .or_else(|| terminal.then(|| terminal_width(&stdout)).flatten())
}

/// The look on a stream that may be standard output or standard error.
pub fn style_on(terminal: bool) -> Style {
    style(
        color_mode(),
        ColorEnv::read(|key| std::env::var_os(key)),
        terminal,
    )
}

fn env_columns() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()?
        .parse()
        .ok()
        .filter(|cols| *cols > 0)
}

fn terminal_width(stdout: &std::io::Stdout) -> Option<usize> {
    rustix::termios::tcgetwinsize(stdout)
        .ok()
        .map(|size| usize::from(size.ws_col))
        .filter(|cols| *cols > 0)
}

fn configured_width() -> Option<usize> {
    match WIDTH.load(Ordering::Relaxed) {
        0 => None,
        cols => Some(cols),
    }
}

fn color_mode() -> ColorMode {
    match COLOR.load(Ordering::Relaxed) {
        1 => ColorMode::Always,
        2 => ColorMode::Never,
        _ => ColorMode::Auto,
    }
}

pub fn configure(color: ColorMode, width: Option<usize>, quiet: bool) {
    let color = match color {
        ColorMode::Auto => 0,
        ColorMode::Always => 1,
        ColorMode::Never => 2,
    };

    COLOR.store(color, Ordering::Relaxed);
    WIDTH.store(width.unwrap_or(0), Ordering::Relaxed);
    QUIET.store(quiet, Ordering::Relaxed);
}

pub fn set_verbose(verbose: bool) {
    VERBOSE.store(verbose, Ordering::Relaxed);
}

pub fn set_json_lines(json: bool) {
    JSON_LINES.store(json, Ordering::Relaxed);
}

pub fn set_json_read(json: bool) {
    JSON_READ.store(json, Ordering::Relaxed);
}

fn json_read() -> bool {
    JSON_READ.load(Ordering::Relaxed)
}

fn json_lines() -> bool {
    JSON_LINES.load(Ordering::Relaxed)
}

pub fn verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

pub fn quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

pub fn unavailable(what: &str, out: &mut dyn Write) -> Exit {
    say_error(out, &format!("not available: {what}"));
    Exit::Unavailable
}

/// An `error:` line of the command's own, a second line of `text` its
/// hint. A read's `--json` keeps it for its object alone, so standard
/// output holds the JSON and standard error nothing but objects.
pub fn say_error(out: &mut dyn Write, text: &str) {
    match sink(json_lines(), json_read()) {
        Sink::Line => {
            write!(out, "{}", said(paint(), text, true)).ok();
        }
        Sink::Text => {
            write!(std::io::stderr(), "{}", said(paint(), text, false)).ok();
        }
        Sink::Kept => {
            keep(Said::new(text, None));
        }
    }
}

/// `error: …` and its `hint: …` line, painted for standard error, as a
/// start problem prints it before any command runs.
pub fn error_text(text: &str) -> String {
    let paint = Paint::new(style_on(std::io::stderr().is_terminal()));
    said(paint, text, false)
}

fn said(paint: Paint, text: &str, json: bool) -> String {
    if json {
        return crate::apply::error_line(text, None);
    }

    match text.split_once('\n') {
        Some((error, hint)) => format!("{}\n{}\n", paint.error(error), paint.hint(hint)),
        None => format!("{}\n", paint.error(text)),
    }
}

/// `what` went wrong, as one line. A program's own words in `raw` follow
/// with `--verbose`, else a hint says how to see them. As with
/// [`say_error`], a read's `--json` keeps it for its object alone.
pub fn failure(out: &mut dyn Write, what: &str, raw: &str) {
    let text = failed_text(paint(), what, raw, verbose(), json_lines());

    match sink(json_lines(), json_read()) {
        Sink::Line => {
            write!(out, "{text}").ok();
        }
        Sink::Text => {
            write!(std::io::stderr(), "{text}").ok();
        }
        Sink::Kept => {
            keep(Said::new(what, Some(raw)));
        }
    }
}

/// Where an error goes: a change's JSON Lines win over a read's `--json`.
const fn sink(json_lines: bool, json_read: bool) -> Sink {
    if json_lines {
        Sink::Line
    } else if json_read {
        Sink::Kept
    } else {
        Sink::Text
    }
}

/// Output that never reached its reader, as one `error:` line on standard
/// error, or under any `--json` kept for its object there.
pub fn lost(kind: std::io::ErrorKind) {
    let text = format!("the output was lost: {kind}");

    match lost_sink(json_lines(), json_read()) {
        Sink::Kept => keep(Said::new(&text, None)),
        Sink::Line | Sink::Text => {
            write!(std::io::stderr(), "{}", error_text(&text)).ok();
        }
    }
}

const fn lost_sink(json_lines: bool, json_read: bool) -> Sink {
    if json_lines || json_read {
        Sink::Kept
    } else {
        Sink::Text
    }
}

fn keep(said: Said) {
    SAID.lock().push(said);
}

/// The objects of the errors kept under `--json`, one line each,
/// for standard error. `exit` is the code the command ends with.
pub fn kept_errors(exit: Exit) -> String {
    let kept = std::mem::take(&mut *SAID.lock());
    kept.iter().map(|said| error_object(said, exit)).collect()
}

/// `{"schema": 1, "error": {...}}` with the exit code as `code` and its
/// name as `reason`.
pub fn error_object(said: &Said, exit: Exit) -> String {
    #[derive(Serialize)]
    struct Coded<'a> {
        #[serde(flatten)]
        said: &'a Said,
        code: u8,
        reason: &'static str,
    }

    #[derive(Serialize)]
    struct Object<'a> {
        error: Coded<'a>,
    }

    json_line(&Object {
        error: Coded {
            said,
            code: exit.code(),
            reason: exit.reason(),
        },
    })
}

fn failed_text(paint: Paint, what: &str, raw: &str, verbose: bool, json: bool) -> String {
    if json {
        return crate::apply::error_line(what, Some(raw));
    }

    failure_text(paint, what, raw, verbose)
}

pub fn failure_text(paint: Paint, what: &str, raw: &str, verbose: bool) -> String {
    let mut text = format!("{}\n", paint.error(what));
    if raw.trim().is_empty() {
        return text;
    }

    if verbose {
        text.push_str(&paint.detail(raw));
    } else {
        let hint = paint.hint("add --verbose to see the program's own words");
        if !hint.is_empty() {
            text.push_str(&hint);
            text.push('\n');
        }
    }

    text
}
