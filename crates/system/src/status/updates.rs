//! Pending updates from the repositories (`checkupdates`) and the AUR
//! (`paru -Qua`). Both print `name old -> new`, one update per line.

#[cfg(test)]
mod tests;

use std::ffi::OsString;

use toolbox_plan::PackageName;

use crate::command;
use crate::error::ProbeError;

/// paru's mark for an update `IgnorePkg` holds back. An update never
/// installs it, so it is not pending.
const IGNORED: &str = "[ignored]";

/// `from` and `to` hold the versions as the program printed them, not a
/// validated value, so they are shown and never passed to a command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingUpdate {
    pub name: PackageName,
    pub from: String,
    pub to: String,
}

pub trait UpdateSource: Send + Sync {
    /// Pending updates from the sync repositories, in the program's order.
    /// Slow: it syncs a copy of the package databases first.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the program,
    /// [`ProbeError::Failed`] when the sync or the program fails.
    fn pending(&self) -> Result<Vec<PendingUpdate>, ProbeError>;
}

pub trait AurUpdateSource: Send + Sync {
    /// Pending updates of foreign packages found in the AUR. Slow: it asks
    /// the AUR over the network.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without an AUR helper or as root,
    /// [`ProbeError::Failed`] when the AUR cannot be asked.
    fn pending_aur(&self) -> Result<Vec<PendingUpdate>, ProbeError>;
}

/// [`UpdateSource`] through `checkupdates` from pacman-contrib.
///
/// It syncs a temporary database (`$TMPDIR/checkup-db-$UID`) under fakeroot, so it
/// needs no root and leaves the system's databases alone. Root may run
/// it too (pacman-contrib 1.13.1 has no root check).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkupdates {
    program: OsString,
}

/// [`AurUpdateSource`] through `paru`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paru {
    program: OsString,
    /// paru must never run as root (see the architecture notes). Off only
    /// for test stand-ins.
    refuse_root: bool,
}

impl Checkupdates {
    /// `checkupdates` from `PATH`.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new() -> Self {
        Self::with_program("checkupdates")
    }

    /// Another program with checkupdates' interface, looked up like a
    /// command.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for Checkupdates {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateSource for Checkupdates {
    /// Exit 0 lists updates, 2 means none, 1 is a failure (no fakeroot,
    /// the sync failed). checkupdates already drops ignored packages.
    fn pending(&self) -> Result<Vec<PendingUpdate>, ProbeError> {
        let finished = command::run(&self.program, &["--nocolor"])?;

        match finished.status.code() {
            Some(0) => Ok(parse_updates(&finished.stdout)),
            Some(2) => Ok(Vec::new()),
            Some(_) | None => Err(finished.failure()),
        }
    }
}

impl Paru {
    /// `paru` from `PATH`, refused as root.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new() -> Self {
        Self {
            program: OsString::from("paru"),
            refuse_root: true,
        }
    }

    /// Another program with paru's `-Qua` interface, looked up like a
    /// command, without the root check.
    #[must_use = "the adapter does nothing until asked"]
    #[cfg(test)]
    pub(crate) fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            refuse_root: false,
        }
    }
}

impl Default for Paru {
    fn default() -> Self {
        Self::new()
    }
}

impl AurUpdateSource for Paru {
    /// Exit 0 lists updates. Exit 1 with nothing printed and no `error:`
    /// means none. paru also exits with 1 when the AUR cannot be reached,
    /// then it says `error:` on stderr.
    fn pending_aur(&self) -> Result<Vec<PendingUpdate>, ProbeError> {
        if self.refuse_root && rustix::process::geteuid().is_root() {
            tracing::warn!("paru is not run as root, AUR updates are not checked");
            return Err(ProbeError::Unavailable);
        }

        let finished = command::run(&self.program, &["-Qua"])?;
        let complained = finished
            .stderr
            .lines()
            .any(|line| line.trim_start().starts_with("error:"));

        match finished.status.code() {
            Some(0) => Ok(parse_updates(&finished.stdout)),
            Some(1) if finished.stdout.trim().is_empty() && !complained => Ok(Vec::new()),
            Some(_) | None => Err(finished.failure()),
        }
    }
}

/// `name old -> new` lines, with optional text after them. Ignored
/// updates, invalid names and other lines are left out.
fn parse_updates(output: &str) -> Vec<PendingUpdate> {
    output.lines().filter_map(parse_update).collect()
}

fn parse_update(line: &str) -> Option<PendingUpdate> {
    let mut words = line.split_whitespace();
    let (name, from, arrow, to) = (words.next()?, words.next()?, words.next()?, words.next()?);

    if arrow != "->" {
        tracing::debug!("not an update: {line}");
        return None;
    }

    if words.any(|word| word == IGNORED) {
        return None;
    }

    match PackageName::new(name) {
        Ok(name) => Some(PendingUpdate {
            name,
            from: from.to_owned(),
            to: to.to_owned(),
        }),
        Err(error) => {
            tracing::debug!("skipped: {error}");
            None
        }
    }
}
