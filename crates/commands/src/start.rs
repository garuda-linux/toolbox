//! Errors before a command runs: clap's and those of the window's start
//! options. When parsing failed there are no parsed arguments, so
//! `--json` is looked for in the raw ones.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::io::Write as _;

use toolbox_cli::Exit;

use crate::out::{Said, error_object, error_text};

pub fn json_asked<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter()
        .take_while(|arg| arg.as_ref() != OsStr::new("--"))
        .any(|arg| arg.as_ref() == OsStr::new("--json"))
}

/// A start option the app does not know: the message on standard error,
/// a second line its hint, as clap prints its own. Under `--json` it is
/// the error object a read prints.
pub fn usage(message: &str) -> Exit {
    let shown = usage_text(message, json_asked(std::env::args_os().skip(1)));
    write!(std::io::stderr(), "{shown}").ok();
    Exit::Usage
}

/// clap's error. Help and the version print as clap prints them.
///
/// A wrong argument is an `error:` line and a `hint:` line as the app's own
/// errors are, under `--json` the error object with code 2.
pub fn parse_failed(error: &clap::Error) -> Exit {
    let rendered = error.render().to_string();
    if !error.use_stderr() || !rendered.starts_with("error: ") {
        error.exit();
    }

    let shown = if json_asked(std::env::args_os().skip(1)) {
        error_object(&clap_said(&rendered), Exit::Usage)
    } else {
        error_text(&clap_text(&rendered))
    };

    write!(std::io::stderr(), "{shown}").ok();
    Exit::Usage
}

fn usage_text(message: &str, json: bool) -> String {
    if json {
        error_object(&Said::new(message, None), Exit::Usage)
    } else {
        error_text(message)
    }
}

fn clap_said(rendered: &str) -> Said {
    Said::new(&clap_text(rendered), None)
}

/// clap's error as the app words its own: the first line the message, the
/// arguments listed under it joined to it, `tip:` lines and the bracketed
/// values the hint. The usage block is left out. Without another hint the
/// `--help` of the command the usage line names is the hint.
fn clap_text(rendered: &str) -> String {
    let mut lines = rendered.lines();
    let first = lines.next().unwrap_or_default();
    let mut message = first.strip_prefix("error: ").unwrap_or(first).to_owned();
    let mut listed: Vec<&str> = Vec::new();
    let mut hints: Vec<&str> = Vec::new();
    let mut command: Option<String> = None;

    for line in lines {
        let trimmed = line.trim();
        if let Some(usage) = line.strip_prefix("Usage: ") {
            command = Some(command_of(usage));
        } else if let Some(tip) = trimmed.strip_prefix("tip: ") {
            hints.push(tip);
        } else if let Some(values) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            hints.push(values);
        } else if command.is_none() && line.starts_with(' ') && !trimmed.is_empty() {
            listed.push(trimmed);
        } else {
            // A blank line, the second line of a usage, and the closing
            // pointer to `--help`, which the hint replaces.
        }
    }

    if !listed.is_empty() {
        message = format!("{message} {}", listed.join(", "));
    }

    let help = command.map(|command| format!("{command} --help"));
    match (hints.is_empty(), help) {
        (false, _) => format!("{message}\n{}", hints.join(" ")),
        (true, Some(help)) => format!("{message}\n{help}"),
        (true, None) => message,
    }
}

/// `garuda-toolbox toggle on` of `garuda-toolbox toggle on <ID>`.
fn command_of(usage: &str) -> String {
    usage
        .split_whitespace()
        .take_while(|word| !word.starts_with(['[', '<', '-']))
        .collect::<Vec<_>>()
        .join(" ")
}
