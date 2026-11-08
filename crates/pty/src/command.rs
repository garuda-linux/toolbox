#[cfg(test)]
mod tests;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{DEFAULT_SIZE, TerminalSize, clamped};

/// Programs pick colours and line drawing from it. Set last, so no extra
/// variable can turn colours off by accident.
const TERM: &str = "xterm-256color";

/// A program to start with [`crate::Session::spawn`]. Built by value:
/// `PtyCommand::new("/usr/bin/pacman").args(["-S", "vlc"]).env_clear()`.
#[derive(Clone, Debug)]
pub struct PtyCommand {
    program: PathBuf,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
    env_clear: bool,
    size: TerminalSize,
}

impl PtyCommand {
    #[must_use = "a command does nothing until spawned"]
    pub fn new<P>(program: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            env: Vec::new(),
            env_clear: false,
            size: DEFAULT_SIZE,
        }
    }

    #[must_use = "the builder returns the changed command"]
    pub fn arg<A>(mut self, arg: A) -> Self
    where
        A: AsRef<OsStr>,
    {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    #[must_use = "the builder returns the changed command"]
    pub fn args<I, A>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

    /// Sets one environment variable. `TERM` is always `xterm-256color`.
    #[must_use = "the builder returns the changed command"]
    pub fn env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env
            .push((key.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }

    /// Starts from an empty environment instead of this process' one. The
    /// root helper does this and sets `PATH` and `LANG` itself.
    #[must_use = "the builder returns the changed command"]
    pub const fn env_clear(mut self) -> Self {
        self.env_clear = true;
        self
    }

    #[must_use = "the builder returns the changed command"]
    pub fn size(mut self, size: TerminalSize) -> Self {
        self.size = clamped(size);
        self
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub const fn terminal_size(&self) -> TerminalSize {
        self.size
    }

    pub(crate) fn to_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);

        if self.env_clear {
            command.env_clear();
        }

        command
            .envs(self.env.iter().map(|(key, value)| (key, value)))
            .env("TERM", TERM);

        command
    }
}
