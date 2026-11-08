//! The report is English whatever the app's language: the people who read
//! it in the forum are not the one who made it.

pub use toolbox_state::diagnostics::{
    Access, Attempt, Collector, Outcome, Program, Ran, Refusal, Selection, Source, UploadFailure,
    full_report, outcome, since_last_pacman_run,
};
