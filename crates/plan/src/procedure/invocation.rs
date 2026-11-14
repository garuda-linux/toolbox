//! One part of a step's procedure, as the helper runs it.

use std::fmt;

use super::{Builtin, Command};
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
        match self {
            // The preview changes nothing, so the command shown is what runs.
            Self::Run(command)
            | Self::RunUnlessItRemovesProtected {
                preview: _,
                run: command,
            } => command.fmt(f),
            Self::RunWithTargetsFrom { query, run } => {
                write!(f, "{run} $({query})  # again until it prints nothing")
            }
            Self::Builtin(builtin) => builtin.fmt(f),
            Self::Always(command)
            | Self::Cleanup {
                command,
                leftover: _,
            } => write!(f, "{command}  # always, even after a failure"),
            Self::Feed { command, input: _ } => {
                write!(f, "{command}  # the secret arrives on its standard input")
            }
        }
    }
}
