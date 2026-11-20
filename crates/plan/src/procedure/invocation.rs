//! One part of a step's procedure, as the helper runs it.

use std::fmt;

use super::{Builtin, Command, Shown};
use crate::SecretInput;
use crate::protocol::Leftover;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    Run(Command),
    /// Runs `query`, then `run` with every line of the query's output as an
    /// extra argument, and again while the query names new targets.
    /// Skipped when the query prints nothing.
    RunWithTargetsFrom {
        query: Command,
        run: Command,
    },
    /// Runs `test`, `pacman -T` of the asked names, then `run` with every
    /// line it prints as an extra argument. Skipped when it prints nothing.
    RunWithUnmetFrom {
        test: Command,
        run: Command,
    },
    /// Runs `preview`, whose lines name every package `run` removes, and
    /// fails the step without running `run` when one is protected.
    RunUnlessItRemovesProtected {
        preview: Command,
        run: Command,
    },
    Builtin(Builtin),
    /// Runs even when an earlier invocation of the same step failed or was
    /// cancelled, and cannot be cancelled itself. For work that must undo a
    /// temporary state, such as starting a service the step stopped. The
    /// step still reports the earlier failure.
    Always(Command),
    /// [`Invocation::Always`] that undoes what a repair set up. When it
    /// fails, the step's outcome names the `leftover`.
    Cleanup {
        command: Command,
        leftover: Leftover,
    },
    /// Runs `command` without a terminal and writes `input` to its
    /// standard input: how a secret reaches a program. `None` while the
    /// secret has not arrived, which fails the step.
    Feed {
        command: Command,
        input: Option<SecretInput>,
    },
}

impl fmt::Display for Invocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_command(f, Shown::WithNotes)?;
        self.why().map_or(Ok(()), |why| write!(f, "  # {why}"))
    }
}

impl Invocation {
    /// The part's command as the Console shows it, without its note.
    pub fn command(&self) -> String {
        fmt::from_fn(|out| self.write_command(out, Shown::Commands)).to_string()
    }

    /// Why the part runs, after `#` in the log and on the Review page.
    pub const fn why(&self) -> Option<&'static str> {
        match self {
            Self::RunWithTargetsFrom { query: _, run: _ } => Some("again until it prints nothing"),
            Self::RunWithUnmetFrom { test: _, run: _ } => Some(
                "only the packages pacman -T names as missing, so an installed one keeps its install reason",
            ),
            Self::Always(_)
            | Self::Cleanup {
                command: _,
                leftover: _,
            } => Some("always, even after a failure"),
            Self::Feed {
                command: _,
                input: _,
            } => Some("the secret arrives on its standard input"),
            Self::Run(_)
            | Self::RunUnlessItRemovesProtected { preview: _, run: _ }
            | Self::Builtin(_) => None,
        }
    }

    /// A builtin's own note stays only with [`Shown::WithNotes`].
    fn write_command(&self, out: &mut fmt::Formatter<'_>, shown: Shown) -> fmt::Result {
        match self {
            // The preview changes nothing, so the command shown is what runs.
            Self::Run(command)
            | Self::RunUnlessItRemovesProtected {
                preview: _,
                run: command,
            }
            | Self::Always(command)
            | Self::Cleanup {
                command,
                leftover: _,
            }
            | Self::Feed { command, input: _ } => fmt::Display::fmt(command, out),
            Self::RunWithTargetsFrom { query: test, run }
            | Self::RunWithUnmetFrom { test, run } => {
                write!(out, "{run} $({test})")
            }
            Self::Builtin(builtin) if shown == Shown::WithNotes => fmt::Display::fmt(builtin, out),
            Self::Builtin(builtin) => builtin.write_command(out, shown),
        }
    }
}
