//! Why a run of the root helper brought no answer. pkexec's and the
//! helper's own words stay data, the window and the command line word the
//! rest.

use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, thiserror::Error)]
pub enum HelperError {
    #[error("the helper of Garuda Toolbox or pkexec is not installed")]
    Missing,
    #[error("cannot start the helper")]
    Start(#[source] Arc<std::io::Error>),
    #[error("the helper gave no answer in time")]
    TimedOut,
    #[error("the helper's answer is too long")]
    TooLong,
    /// pkexec itself cannot work here, with its own words.
    #[error("pkexec cannot run: {words}")]
    Pkexec { words: String },
    /// The helper failed, with the start of what it printed on stderr.
    #[error("the helper ended with {}: {words}", Ending(*code))]
    Ended { code: Option<i32>, words: String },
    /// Its answer is not what the caller asked for.
    #[error("the helper's answer is unreadable")]
    Unreadable,
    /// The task that waited for the helper ended first.
    #[error("the read stopped before the helper answered")]
    Stopped,
}

/// A start error compares by its kind, as `std::io::Error` has no
/// equality of its own.
impl PartialEq for HelperError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Start(one), Self::Start(two)) => one.kind() == two.kind(),
            (Self::Pkexec { words: one }, Self::Pkexec { words: two }) => one == two,
            (
                Self::Ended { code, words },
                Self::Ended {
                    code: other_code,
                    words: other_words,
                },
            ) => code == other_code && words == other_words,
            (Self::Missing, Self::Missing)
            | (Self::TimedOut, Self::TimedOut)
            | (Self::TooLong, Self::TooLong)
            | (Self::Unreadable, Self::Unreadable)
            | (Self::Stopped, Self::Stopped) => true,
            _ => false,
        }
    }
}

impl Eq for HelperError {}

struct Ending(Option<i32>);

impl fmt::Display for Ending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(code) => write!(f, "exit code {code}"),
            None => f.write_str("a signal"),
        }
    }
}
