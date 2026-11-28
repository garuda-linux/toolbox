//! What the Diagnostics page shows, as plain state: codes and the report
//! in its two forms. The UI picks the words.

#[cfg(test)]
mod tests;

use std::time::Instant;

use jiff::Timestamp;
use toolbox_pty::strip_ansi;
use toolbox_state::diagnostics::{Collector, Failure, Outcome, Redacted, Refusal, Selection};

use super::service::DiagnosticsNote;
use crate::share::{ShareNote, Upload};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Running,
    Report,
    Unavailable,
    Refused(Refusal),
    Failed,
    Stopped,
}

/// What a note changed, so the app redraws only that: the report can be
/// megabytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Changed {
    Status,
    Report { ready: bool },
    Upload { done: bool },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Page {
    pub state: State,
    pub selection: Option<Selection>,
    pub part: Option<Collector>,
    /// Its place among the collectors of a full report, from 1.
    pub step: i32,
    /// The report as plain text: what is copied.
    pub report: String,
    /// The report as the programs printed it, colours included. Only ever
    /// shown, never copied or sent.
    pub raw: String,
    /// The plain report as it may leave the machine: what the upload
    /// question shows and the upload sends. `None` until it is ready.
    pub shareable: Option<Redacted>,
    /// Why the collection failed, `None` unless the state is `Failed`.
    pub problem: Option<Failure>,
    pub share: Upload,
    /// When the shown report came in. The owner of the clock stamps it, so
    /// a page stays plain state.
    pub collected: Option<Timestamp>,
}

impl State {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Report => "report",
            Self::Unavailable => "unavailable",
            Self::Refused(refusal) => refusal.code(),
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }
}

impl Page {
    pub fn apply(&mut self, note: DiagnosticsNote) -> Changed {
        match note {
            DiagnosticsNote::Started(selection) => {
                self.state = State::Running;
                self.selection = Some(selection);
                self.clear_result();

                // The last link belongs to the report this run replaces.
                self.share.clear();
                Changed::Report { ready: false }
            }
            DiagnosticsNote::Collecting(collector) => {
                self.part = Some(collector);
                self.step = step_of(collector);
                Changed::Status
            }
            DiagnosticsNote::Finished(selection, outcome) => {
                self.selection = Some(selection);
                self.clear_result();
                self.finish(outcome)
            }
            DiagnosticsNote::Stopped(selection) => {
                self.selection = Some(selection);
                self.clear_result();
                self.state = State::Stopped;
                Changed::Report { ready: false }
            }
            DiagnosticsNote::Shareable(redacted) => {
                self.shareable = Some(redacted);
                Changed::Status
            }
            DiagnosticsNote::Cancelled => {
                self.state = State::Idle;
                self.clear_result();
                Changed::Status
            }
            DiagnosticsNote::UploadStarted => Changed::Upload {
                done: self.share.apply(ShareNote::Started),
            },
            DiagnosticsNote::Uploaded(result) => Changed::Upload {
                done: self.share.apply(ShareNote::Uploaded(result)),
            },
        }
    }

    pub fn clear_upload(&mut self) {
        self.share.clear();
    }

    pub(crate) fn start_upload(&mut self) {
        self.share.start();
    }

    fn show(&mut self, raw: String) {
        let started = Instant::now();

        // Everything that leaves the page (the clipboard, the upload and
        // its confirm dialog) gets plain text.
        self.report = strip_ansi(&raw);

        tracing::debug!(
            "a report of {} bytes: plain text of {} bytes after {} µs",
            raw.len(),
            self.report.len(),
            started.elapsed().as_micros()
        );
        self.raw = raw;
    }

    fn finish(&mut self, outcome: Outcome) -> Changed {
        match outcome {
            Outcome::Report(text) => {
                self.state = State::Report;
                self.show(text);
                Changed::Report { ready: true }
            }
            Outcome::Unavailable => {
                self.state = State::Unavailable;
                Changed::Report { ready: false }
            }
            Outcome::Refused(refusal) => {
                self.state = State::Refused(refusal);
                Changed::Report { ready: false }
            }
            Outcome::Failed(reason) => {
                self.state = State::Failed;
                self.problem = Some(reason);
                Changed::Report { ready: false }
            }
        }
    }

    fn clear_result(&mut self) {
        self.report.clear();
        self.raw.clear();
        self.shareable = None;
        self.problem = None;
        self.collected = None;
        self.part = None;
        self.step = 0_i32;
    }
}

fn step_of(collector: Collector) -> i32 {
    Collector::ALL
        .iter()
        .position(|one| *one == collector)
        .and_then(|index| i32::try_from(index).ok())
        .map_or(0, |index| index.saturating_add(1))
}
