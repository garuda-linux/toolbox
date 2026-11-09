//! One JSON object per line: the app writes a [`Request`] and [`Answer`]s to the helper's stdin, the helper writes [`Event`]s to stdout.

mod read;
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Deserializer, Serialize};

pub use self::read::{ProgramAnswer, ReadKind};
use crate::shown::Shown;
use crate::{PackageName, Password, RootAction, Step, StepId, protection};

/// Bumped whenever a message changes shape. App and helper ship together,
/// so a mismatch means a broken install or a stale helper.
pub const PROTOCOL_VERSION: u32 = 19;

/// The most text one [`Answer::Input`] carries. The sender splits a longer paste.
pub const MAX_INPUT_BYTES: usize = 64 << 10;

/// Longest line either side reads. Program output is sent in chunks well
/// below it. A config file of `MAX_CONFIG_BYTES` fits whatever it holds:
/// a control character grows sixfold as a JSON escape.
pub const MAX_LINE_BYTES: usize = 8 << 20;

pub const MAX_STEPS: usize = 256;

/// Room for a decoder's message about one field, not for the request.
const MALFORMED_CHARACTERS: usize = 160;

/// Only the end of a longer output is kept, since the newest lines matter most.
pub const MAX_READ_OUTPUT_BYTES: usize = 2 << 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Run,
    /// Reports every step as skipped without running it. Kept in the request because pkexec drops the caller's environment.
    DryRun,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub version: u32,
    pub mode: Mode,
    pub steps: Vec<Step<RootAction>>,
}

#[derive(Deserialize)]
struct Head {
    version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RequestError {
    /// Quotes the request, so only its start goes back to the app and into the log.
    #[error("the request is malformed: {}", Shown::up_to(.0, MALFORMED_CHARACTERS))]
    Malformed(String),
    #[error("protocol version {got} does not match the helper's {expected}")]
    VersionMismatch { expected: u32, got: u32 },
    #[error("the request has no steps")]
    NoSteps,
    #[error("the request has more than {max} steps")]
    TooManySteps { max: usize },
    #[error("step {0} appears twice")]
    DuplicateStep(StepId),
    #[error("the secret for step {0} is missing or misplaced")]
    MissingSecret(StepId),
    /// The app refuses this first. A request that still names one did not
    /// come through the app's guards.
    #[error("step {step} removes the protected package {package}")]
    Protected { step: StepId, package: PackageName },
}

impl Request {
    #[must_use = "a request does nothing until it is sent"]
    pub const fn new(mode: Mode, steps: Vec<Step<RootAction>>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            mode,
            steps,
        }
    }

    /// Names inside the actions are checked while decoding, so a request that parses is safe to run.
    ///
    /// # Errors
    ///
    /// [`RequestError`] for a malformed line or a request that breaks a
    /// rule of [`Request::validate`].
    pub fn parse(line: &str) -> Result<Self, RequestError> {
        let malformed = |error: DecodeError| RequestError::Malformed(error.to_string());

        // Before the full decode: another version's request may hold
        // actions unknown here, and the version is the reason to report.
        let Head { version } = decode(line).map_err(malformed)?;

        if version != PROTOCOL_VERSION {
            return Err(RequestError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: version,
            });
        }

        let request: Self = decode(line).map_err(malformed)?;

        request.validate()?;
        Ok(request)
    }

    /// # Errors
    ///
    /// [`RequestError`] for a version mismatch, no steps, too many steps,
    /// a step id used twice or the removal of a protected package.
    pub fn validate(&self) -> Result<(), RequestError> {
        if self.version != PROTOCOL_VERSION {
            return Err(RequestError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: self.version,
            });
        }

        if self.steps.is_empty() {
            return Err(RequestError::NoSteps);
        }

        if self.steps.len() > MAX_STEPS {
            return Err(RequestError::TooManySteps { max: MAX_STEPS });
        }

        let mut seen = HashSet::new();
        if let Some(step) = self.steps.iter().find(|step| !seen.insert(step.id)) {
            return Err(RequestError::DuplicateStep(step.id));
        }

        self.steps.iter().try_for_each(refuse_protected)
    }

    /// A dry run needs none: nothing runs.
    pub fn secret_steps(&self) -> Vec<StepId> {
        match self.mode {
            Mode::DryRun => Vec::new(),
            Mode::Run => self
                .steps
                .iter()
                .filter(|step| step.action.needs_secret())
                .map(|step| step.id)
                .collect(),
        }
    }

    /// # Errors
    ///
    /// [`RequestError::MissingSecret`] when no such step takes a secret.
    pub fn attach_secret(&mut self, step: StepId, secret: Password) -> Result<(), RequestError> {
        let taken = self
            .steps
            .iter_mut()
            .find(|candidate| candidate.id == step)
            .is_some_and(|candidate| candidate.action.attach_secret(secret));

        if taken {
            Ok(())
        } else {
            Err(RequestError::MissingSecret(step))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Event {
    Accepted,
    Rejected {
        reason: String,
    },
    Started {
        step: StepId,
    },
    /// Raw terminal output, decoded as lossy UTF-8.
    Output {
        step: StepId,
        data: String,
    },
    Finished {
        step: StepId,
        outcome: Outcome,
    },
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Outcome {
    Succeeded,
    Failed {
        cause: Failure,
    },
    /// Also for a dry run, or an empty target query.
    Skipped,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Failure {
    ExitCode {
        code: i32,
    },
    Signal {
        signal: i32,
    },
    /// No program ended: it did not start, a builtin failed or a check
    /// refused. The step's output says why.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSize {
    pub columns: u16,
    pub rows: u16,
}

impl From<(u16, u16)> for TerminalSize {
    fn from((columns, rows): (u16, u16)) -> Self {
        Self { columns, rows }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Answer {
    Input {
        #[serde(deserialize_with = "capped_input")]
        data: String,
    },
    Resize {
        size: TerminalSize,
    },
    Cancel,
    /// An [`Answer::Cancel`] after it still stops the running step.
    Finish,
}

impl Answer {
    /// Splits an input longer than [`MAX_INPUT_BYTES`] into several parts, cut between characters, which the program reads as one stream.
    pub fn parts(self) -> Vec<Self> {
        let Self::Input { data } = &self else {
            return vec![self];
        };

        if data.len() <= MAX_INPUT_BYTES {
            return vec![self];
        }

        let mut parts = Vec::new();
        let mut rest = data.as_str();

        while !rest.is_empty() {
            let end = rest.floor_char_boundary(MAX_INPUT_BYTES);
            let (part, tail) = rest.split_at(end);

            parts.push(Self::Input {
                data: part.to_owned(),
            });
            rest = tail;
        }

        parts
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("line of {length} bytes exceeds the limit of {limit}")]
    TooLong { length: usize, limit: usize },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Never quotes the text: it may be a password typed at a prompt.
fn capped_input<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let data = String::deserialize(deserializer)?;

    if data.len() > MAX_INPUT_BYTES {
        return Err(D::Error::custom(format_args!(
            "input of {} bytes exceeds the limit of {MAX_INPUT_BYTES}",
            data.len()
        )));
    }

    Ok(data)
}

/// # Errors
///
/// Only when `message` cannot be serialized, which the types here never do.
pub fn encode<T>(message: &T) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    let mut line = serde_json::to_string(message)?;
    line.push('\n');
    Ok(line)
}

/// # Errors
///
/// [`DecodeError`] for an overlong line or invalid JSON.
pub fn decode<T>(line: &str) -> Result<T, DecodeError>
where
    T: DeserializeOwned,
{
    decode_within(line, MAX_LINE_BYTES)
}

fn decode_within<T>(line: &str, limit: usize) -> Result<T, DecodeError>
where
    T: DeserializeOwned,
{
    if line.len() > limit {
        return Err(DecodeError::TooLong {
            length: line.len(),
            limit,
        });
    }

    Ok(serde_json::from_str(line)?)
}

fn refuse_protected(step: &Step<RootAction>) -> Result<(), RequestError> {
    let RootAction::RemovePackages { packages } = &step.action else {
        return Ok(());
    };

    packages
        .iter()
        .find(|package| protection(package).is_some())
        .map_or(Ok(()), |package| {
            Err(RequestError::Protected {
                step: step.id,
                package: package.clone(),
            })
        })
}
