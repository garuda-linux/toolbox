//! The exact programs an action runs. The helper executes these, and the
//! Review page shows them, so both always agree.

mod builtin;
mod initramfs;
mod invocation;
mod notes;
mod program;
#[cfg(test)]
mod tests;
mod uki;

use std::fmt;

pub use builtin::Builtin;
pub use initramfs::{InitramfsRoot, initramfs_rebuilds};
pub use invocation::Invocation;
pub(crate) use notes::Shown;
pub use program::Program;
pub use uki::{in_root, relaxed, uki_rebuilds};

/// Where [`Builtin::InstallMirrorlist`] writes, relative to `/`.
pub const CHAOTIC_MIRRORLIST: &str = "etc/pacman.d/chaotic-mirrorlist";

/// The lock pacman holds while it runs, relative to `/`.
pub const DATABASE_LOCK: &str = "var/lib/pacman/db.lck";

/// The login shells the system allows, relative to `/`.
pub const SHELLS: &str = "etc/shells";

/// The account files the account guards read, relative to `/`.
pub const PASSWD: &str = "etc/passwd";
pub const GROUP: &str = "etc/group";

/// `UID_MIN` and `GID_MIN` of `/etc/login.defs`: ids below belong to the
/// system.
pub const FIRST_USER_ID: u32 = 1000;

/// One program run: argv plus the variables it gets on top of the
/// helper's clean environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    program: Program,
    args: Vec<String>,
    env: Vec<(&'static str, &'static str)>,
}

impl Command {
    #[must_use = "a command does nothing until it is run"]
    pub const fn new(program: Program) -> Self {
        Self {
            program,
            args: Vec::new(),
            env: Vec::new(),
        }
    }

    #[must_use = "builder methods return the changed command"]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use = "builder methods return the changed command"]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    #[must_use = "builder methods return the changed command"]
    pub fn env(mut self, key: &'static str, value: &'static str) -> Self {
        self.env.push((key, value));
        self
    }

    pub const fn program(&self) -> Program {
        self.program
    }

    pub fn arguments(&self) -> &[String] {
        &self.args
    }

    pub fn variables(&self) -> &[(&'static str, &'static str)] {
        &self.env
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in &self.env {
            write!(f, "{key}={value} ")?;
        }

        f.write_str(self.program.name())?;
        for arg in &self.args {
            write!(f, " {}", quoted(arg))?;
        }

        Ok(())
    }
}

/// Single quotes for display when an argument holds anything a shell would
/// treat specially. Validated names never need it.
fn quoted(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_./=:@+,".contains(character));

    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}
