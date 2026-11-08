//! Long text through a pager, as git does: lists and shows on a terminal
//! go through `$GARUDA_TOOLBOX_PAGER`, else `$PAGER`, else `less -FRX`,
//! which quits at once when the text fits one screen and keeps colours.
//! The pager gets `LESS=FRX` and `LV=-c` when they are unset, so a bare
//! `PAGER=less` keeps the colours too.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::io::IsTerminal as _;
use std::process::{Child, ChildStdin, Stdio};

use tokio::runtime::Runtime;
use tokio::signal::unix::{Signal, SignalKind, signal};
use toolbox_cli::args::{Command, ConfigsCommand};

use crate::reads::read_json;

const OWN: &str = "GARUDA_TOOLBOX_PAGER";

const LESS: [&str; 2] = ["less", "-FRX"];

pub struct Pager {
    child: Child,
    /// Ctrl+C belongs to the pager while it runs, so the terminal is not
    /// handed back to the shell under it.
    _interrupt: Option<Signal>,
}

impl Pager {
    /// The pager `command` should print through, `None` to print directly:
    /// no terminal, `--no-pager`, a pager turned off or one that did not
    /// start.
    pub fn open(
        command: &Command,
        no_pager: bool,
        runtime: &Runtime,
    ) -> Option<(Self, ChildStdin)> {
        let argv = chosen(
            command,
            no_pager,
            std::io::stdout().is_terminal(),
            std::env::var_os(OWN).as_deref().map(text).as_deref(),
            std::env::var_os("PAGER").as_deref().map(text).as_deref(),
        )?;

        let (program, words) = argv.split_first()?;
        let mut child = std::process::Command::new(program)
            .args(words)
            .envs(defaults(
                std::env::var_os("LESS").as_deref(),
                std::env::var_os("LV").as_deref(),
            ))
            .stdin(Stdio::piped())
            .spawn()
            .inspect_err(|error| tracing::info!("the pager {program} did not start: {error}"))
            .ok()?;

        let input = child.stdin.take()?;
        let interrupt = {
            let _entered = runtime.enter();
            signal(SignalKind::interrupt()).ok()
        };

        Some((
            Self {
                child,
                _interrupt: interrupt,
            },
            input,
        ))
    }

    /// Waits until the reader quits. Its stdin must be closed first.
    pub fn wait(mut self) {
        if let Err(error) = self.child.wait() {
            tracing::info!("the pager could not be waited for: {error}");
        }
    }
}

pub fn chosen(
    command: &Command,
    no_pager: bool,
    terminal: bool,
    own: Option<&str>,
    pager: Option<&str>,
) -> Option<Vec<String>> {
    if no_pager || !terminal || !pages(command) {
        return None;
    }

    argv(own, pager)
}

/// The words of the first variable set, as argv without a shell. Empty or
/// `cat` turns the pager off.
pub fn argv(own: Option<&str>, pager: Option<&str>) -> Option<Vec<String>> {
    let words: Vec<String> = own.or(pager).map_or_else(
        || LESS.iter().map(|word| (*word).to_owned()).collect(),
        |value| value.split_whitespace().map(str::to_owned).collect(),
    );

    match words.as_slice() {
        [] => None,
        [only] if only == "cat" => None,
        _ => Some(words),
    }
}

/// The variables the pager gets when they are unset, as git sets them:
/// `$PAGER` may name a bare `less` or `lv`, and the output keeps its
/// colours on a terminal.
pub fn defaults(less: Option<&OsStr>, lv: Option<&OsStr>) -> Vec<(&'static str, &'static str)> {
    [("LESS", "FRX", less), ("LV", "-c", lv)]
        .into_iter()
        .filter(|(_, _, set)| set.is_none())
        .map(|(name, value, _)| (name, value))
        .collect()
}

/// A variable's text. A value that is no text is set but unusable, so it
/// turns the pager off rather than falling back.
pub fn text(raw: &OsStr) -> String {
    raw.to_str().map(str::to_owned).unwrap_or_default()
}

/// Whether `command` prints text worth paging: a list or a show. Never
/// JSON, a question, a plan or a program's running output.
pub const fn pages(command: &Command) -> bool {
    // `status --watch` redraws the screen, which a pager would hold back,
    // and `doctor` prints as `status` does.
    if matches!(command, Command::Status(_) | Command::Doctor(_)) {
        return false;
    }

    matches!(command, Command::Configs(ConfigsCommand::Show(_)))
        || matches!(read_json(command), Some(false))
}
