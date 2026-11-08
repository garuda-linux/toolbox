//! `garuda-health --json` prints one object with a list per level:
//!
//! ```json
//! {"INFO": [], "WARNING": [{"description": "…", "fix_available": true}], "ERROR": []}
//! ```
//!
//! Only this shape is known. Unknown keys and fields are ignored, so a newer garuda-health still reads.

#[cfg(test)]
mod tests;

use std::ffi::OsString;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::command;
use crate::error::ProbeError;

/// The report's keys, most severe first: the order of the issues.
const LEVELS: [(&str, HealthLevel); 3] = [
    ("ERROR", HealthLevel::Error),
    ("WARNING", HealthLevel::Warning),
    ("INFO", HealthLevel::Info),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HealthLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthIssue {
    pub level: HealthLevel,
    /// garuda-health's own text, English. Shown as program output, not
    /// translated.
    pub message: String,
    pub fix_available: bool,
}

pub trait HealthSource: Send + Sync {
    /// Every issue, most severe first. Empty when all is well.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the program,
    /// [`ProbeError::Failed`] when it fails or prints no report.
    fn issues(&self) -> Result<Vec<HealthIssue>, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GarudaHealth {
    program: OsString,
}

#[derive(Deserialize)]
struct Entry {
    description: Option<String>,
    fix_available: Option<bool>,
}

impl GarudaHealth {
    #[must_use = "the adapter does nothing until asked"]
    pub fn new() -> Self {
        Self::with_program("garuda-health")
    }

    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for GarudaHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthSource for GarudaHealth {
    /// The exit code is no verdict (assumed: non-zero when it finds
    /// issues). A report on stdout counts either way. Without one, a non-zero exit is a failure.
    fn issues(&self) -> Result<Vec<HealthIssue>, ProbeError> {
        let finished = command::run(&self.program, &["--json"])?;

        if finished.stdout.trim().is_empty() && !finished.status.success() {
            return Err(finished.failure());
        }

        parse_health(&finished.stdout)
    }
}

/// Empty output is a healthy system, and entries without a description are left out.
fn parse_health(report: &str) -> Result<Vec<HealthIssue>, ProbeError> {
    if report.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut levels: Map<String, Value> =
        serde_json::from_str(report).map_err(|error| invalid(&error))?;

    let mut issues = Vec::new();

    for (key, level) in LEVELS {
        let Some(list) = levels.remove(key) else {
            continue;
        };

        let entries: Vec<Entry> = serde_json::from_value(list).map_err(|error| invalid(&error))?;

        issues.extend(entries.into_iter().filter_map(|entry| {
            let message = entry.description?.trim().to_owned();

            (!message.is_empty()).then(|| HealthIssue {
                level,
                message,
                fix_available: entry.fix_available.unwrap_or_default(),
            })
        }));
    }

    Ok(issues)
}

fn invalid(error: &serde_json::Error) -> ProbeError {
    ProbeError::Failed(format!("garuda-health --json: {error}"))
}
