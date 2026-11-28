//! Why a collection failed. The report carries the English sentence of
//! each code, the window words the code in the user's language.

use std::fmt::{self, Display};

use super::report::Refusal;

/// Why a collection failed, as a closed code with its data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// No program was left to run.
    NoProgramRan,
    /// The program ended with this exit code and printed no error text.
    ExitCode(i32),
    /// A signal ended the program and it printed no error text.
    Signal,
    /// The error text the program printed.
    Stderr(String),
    /// The program could not be started.
    Start { program: String, error: String },
    /// The program ran past its deadline.
    TimedOut { program: String, seconds: u64 },
    /// The program printed more than the report keeps.
    TooLarge { program: String },
    /// A file the collection reads could not be read.
    Read { path: String, error: String },
    /// The root helper failed.
    Helper(HelperFault),
    /// The helper's answer is not one the app can read.
    Answer(AnswerFault),
    /// A later program of the collection is not installed.
    NotInstalled,
    /// A later program of the collection was refused.
    Refused(Refusal),
}

/// What the root helper could not do, as `HelperError` words it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelperFault {
    Missing,
    /// The helper could not be started, with the system's words for why.
    Start {
        error: String,
    },
    TimedOut,
    TooLong,
    /// pkexec itself cannot run, with its own words.
    Pkexec {
        words: String,
    },
    /// The helper failed, with the start of what it printed on stderr.
    Ended {
        code: Option<i32>,
        words: String,
    },
    Unreadable,
    Stopped,
}

/// Why the helper's answer line is not read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnswerFault {
    TooLong {
        length: usize,
        limit: usize,
    },
    /// The words the JSON decoder gave.
    Json(String),
}

impl Failure {
    /// The same failure with `convert` applied to every text the program
    /// or the helper printed, such as the colour codes they carry.
    #[must_use = "the converted failure is the result"]
    pub fn map_text(self, convert: fn(&str) -> String) -> Self {
        match self {
            Self::Stderr(text) => Self::Stderr(convert(&text)),
            Self::Start { program, error } => Self::Start {
                program,
                error: convert(&error),
            },
            Self::Read { path, error } => Self::Read {
                path,
                error: convert(&error),
            },
            Self::Helper(fault) => Self::Helper(fault.map_text(convert)),
            Self::Answer(AnswerFault::Json(text)) => {
                Self::Answer(AnswerFault::Json(convert(&text)))
            }
            Self::Answer(
                fault @ AnswerFault::TooLong {
                    length: _,
                    limit: _,
                },
            ) => Self::Answer(fault),
            Self::NoProgramRan => Self::NoProgramRan,
            Self::ExitCode(code) => Self::ExitCode(code),
            Self::Signal => Self::Signal,
            Self::TimedOut { program, seconds } => Self::TimedOut { program, seconds },
            Self::TooLarge { program } => Self::TooLarge { program },
            Self::NotInstalled => Self::NotInstalled,
            Self::Refused(refusal) => Self::Refused(refusal),
        }
    }
}

impl HelperFault {
    fn map_text(self, convert: fn(&str) -> String) -> Self {
        match self {
            Self::Start { error } => Self::Start {
                error: convert(&error),
            },
            Self::Pkexec { words } => Self::Pkexec {
                words: convert(&words),
            },
            Self::Ended { code, words } => Self::Ended {
                code,
                words: convert(&words),
            },
            Self::Missing => Self::Missing,
            Self::TimedOut => Self::TimedOut,
            Self::TooLong => Self::TooLong,
            Self::Unreadable => Self::Unreadable,
            Self::Stopped => Self::Stopped,
        }
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProgramRan => f.write_str("No program ran."),
            Self::ExitCode(code) => write!(f, "Exit code: {code}"),
            Self::Signal => f.write_str("Ended by a signal."),
            Self::Stderr(text) => f.write_str(text),
            Self::Start { program, error } => {
                write!(f, "{program} could not be started: {error}")
            }
            Self::TimedOut { program, seconds } => {
                write!(f, "{program} took longer than {seconds} seconds")
            }
            Self::TooLarge { program } => write!(f, "{program} printed too much"),
            Self::Read { path, error } => write!(f, "{path}: {error}"),
            Self::Helper(fault) => write!(f, "{fault}"),
            Self::Answer(fault) => write!(
                f,
                "The answer of the helper of Garuda Toolbox was not understood: {fault}"
            ),
            Self::NotInstalled => f.write_str("The next program is not installed."),
            Self::Refused(refusal) => f.write_str(refusal.report_note()),
        }
    }
}

impl Display for HelperFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("the helper of Garuda Toolbox or pkexec is not installed"),
            Self::Start { error } => write!(f, "cannot start the helper: {error}"),
            Self::TimedOut => f.write_str("the helper gave no answer in time"),
            Self::TooLong => f.write_str("the helper's answer is too long"),
            Self::Pkexec { words } => write!(f, "pkexec cannot run: {words}"),
            Self::Ended {
                code: Some(code),
                words,
            } => write!(f, "the helper ended with exit code {code}: {words}"),
            Self::Ended { code: None, words } => {
                write!(f, "the helper ended with a signal: {words}")
            }
            Self::Unreadable => f.write_str("the helper's answer is unreadable"),
            Self::Stopped => f.write_str("the read stopped before the helper answered"),
        }
    }
}

impl Display for AnswerFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { length, limit } => {
                write!(f, "line of {length} bytes exceeds the limit of {limit}")
            }
            Self::Json(text) => f.write_str(text),
        }
    }
}
