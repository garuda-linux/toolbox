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

/// clap's error. Help, the version and text output print as clap prints
/// them and end the process. A wrong argument under `--json` becomes the
/// error object with code 2.
pub fn parse_failed(error: &clap::Error) -> Exit {
    if !error.use_stderr() || !json_asked(std::env::args_os().skip(1)) {
        error.exit();
    }

    let said = clap_said(&error.render().to_string());
    write!(std::io::stderr(), "{}", error_object(&said, Exit::Usage)).ok();
    Exit::Usage
}

fn usage_text(message: &str, json: bool) -> String {
    if json {
        error_object(&Said::new(message, None), Exit::Usage)
    } else {
        error_text(message)
    }
}

/// clap's first line is the message, its `tip:` lines the hint. The usage
/// line under them is left out, `--help` shows it.
fn clap_said(rendered: &str) -> Said {
    let mut lines = rendered.lines();
    let first = lines.next().unwrap_or_default();
    let message = first.strip_prefix("error: ").unwrap_or(first);
    let tips: Vec<&str> = lines
        .filter_map(|line| line.trim().strip_prefix("tip: "))
        .collect();

    if tips.is_empty() {
        Said::new(message, None)
    } else {
        Said::new(&format!("{message}\n{}", tips.join(" ")), None)
    }
}
