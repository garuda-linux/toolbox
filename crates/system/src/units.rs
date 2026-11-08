//! Enabled and masked systemd units of the system and of the user's
//! session, from `systemctl list-unit-files`.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ffi::OsString;

use toolbox_plan::UnitName;
use toolbox_state::parts::{EnabledUnits, EnabledUserUnits, MaskedUnits, MaskedUserUnits};

use crate::error::ProbeError;
use crate::{command, names};

/// Only `enabled`, not `enabled-runtime`: runtime links live in `/run` and
/// vanish at the next boot. Counting them would let a queued enable drop
/// out as done while the unit stays off after a reboot.
const ENABLED: &str = "enabled";

/// Only `masked`, not `masked-runtime`, for the same reason as [`ENABLED`].
const MASKED: &str = "masked";

const USER: [&str; 1] = ["--user"];

const FAILED_ARGS: [&str; 5] = [
    "list-units",
    "--state=failed",
    "--no-legend",
    "--no-pager",
    "--plain",
];

/// Answers which units start on their own and which can never start.
pub trait UnitSource: Send + Sync {
    /// Units enabled for the whole system.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without systemd,
    /// [`ProbeError::Failed`] when the manager gives no answer.
    fn enabled_units(&self) -> Result<EnabledUnits, ProbeError>;

    /// Units enabled in the user's own service manager.
    ///
    /// # Errors
    ///
    /// As for [`UnitSource::enabled_units`].
    fn enabled_user_units(&self) -> Result<EnabledUserUnits, ProbeError>;

    /// Units masked for the whole system.
    ///
    /// # Errors
    ///
    /// As for [`UnitSource::enabled_units`].
    fn masked_units(&self) -> Result<MaskedUnits, ProbeError>;

    /// Units masked in the user's own service manager.
    ///
    /// # Errors
    ///
    /// As for [`UnitSource::enabled_units`].
    fn masked_user_units(&self) -> Result<MaskedUserUnits, ProbeError>;
}

/// Answers which system units stopped with an error.
pub trait FailedUnitSource: Send + Sync {
    /// Units of the whole system in the `failed` state, sorted by name.
    ///
    /// # Errors
    ///
    /// As for [`UnitSource::enabled_units`].
    fn failed_units(&self) -> Result<Vec<UnitName>, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Systemctl {
    program: OsString,
}

impl Systemctl {
    /// `systemctl` from `PATH`.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new() -> Self {
        Self::with_program("systemctl")
    }

    /// Another program with systemctl's interface, looked up like a command.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
        }
    }

    fn list(&self, scope: &[&str], state: &str) -> Result<BTreeSet<UnitName>, ProbeError> {
        let state_flag = format!("--state={state}");
        let args = list_args(scope, &state_flag);

        command::output(&self.program, &args).map(|output| parse_listed(&output, state))
    }
}

impl Default for Systemctl {
    fn default() -> Self {
        Self::new()
    }
}

impl UnitSource for Systemctl {
    fn enabled_units(&self) -> Result<EnabledUnits, ProbeError> {
        self.list(&[], ENABLED).map(EnabledUnits)
    }

    fn enabled_user_units(&self) -> Result<EnabledUserUnits, ProbeError> {
        self.list(&USER, ENABLED).map(EnabledUserUnits)
    }

    fn masked_units(&self) -> Result<MaskedUnits, ProbeError> {
        self.list(&[], MASKED).map(MaskedUnits)
    }

    fn masked_user_units(&self) -> Result<MaskedUserUnits, ProbeError> {
        self.list(&USER, MASKED).map(MaskedUserUnits)
    }
}

impl FailedUnitSource for Systemctl {
    fn failed_units(&self) -> Result<Vec<UnitName>, ProbeError> {
        command::output(&self.program, &FAILED_ARGS)
            .map(|output| parse_failed(&output).into_iter().collect())
    }
}

/// Lines of `UNIT LOAD ACTIVE SUB DESCRIPTION`. The active state is
/// checked again, so a summary line does not count as a unit.
fn parse_failed(output: &str) -> BTreeSet<UnitName> {
    names::valid(output.lines().filter_map(|line| {
        let mut columns = line.split_whitespace();
        let unit = columns.next()?;

        (columns.nth(1) == Some("failed")).then_some(unit)
    }))
}

/// The arguments of one listing. `state_flag` is `--state=<state>`.
fn list_args<'a>(scope: &[&'a str], state_flag: &'a str) -> Vec<&'a str> {
    scope
        .iter()
        .copied()
        .chain([
            "list-unit-files",
            state_flag,
            "--no-legend",
            "--no-pager",
            "--plain",
        ])
        .collect()
}

/// Lines of `UNIT STATE [PRESET]`. The state is checked again, so a legend
/// or summary line that slips through does not count as a unit.
fn parse_listed(output: &str, state: &str) -> BTreeSet<UnitName> {
    names::valid(output.lines().filter_map(|line| {
        let mut columns = line.split_whitespace();
        let unit = columns.next()?;

        (columns.next() == Some(state)).then_some(unit)
    }))
}
