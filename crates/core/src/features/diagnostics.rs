//! The report is English whatever the app's language: the people who read
//! it in the forum are not the one who made it.

pub use toolbox_state::diagnostics::{
    Access, Attempt, Collector, Known, Outcome, Program, Ran, Redacted, RedactedKind, Refusal,
    Selection, Source, UploadFailure, full_report, outcome, redact, since_last_pacman_run,
};
