//! The Diagnostics page's collectors against the running system: programs
//! as the user, fixed reads of the root helper where the user's rights do
//! not reach, and pacman's log.

mod access;
pub mod known;
#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

pub(crate) use access::{access, pacman_log};
use known::KnownRoots;
use toolbox_exec::{CaptureError, Captured, Environment, Policy, capture, locate_program};
use toolbox_plan::protocol::{DecodeError, MAX_READ_OUTPUT_BYTES, ProgramAnswer, ReadKind};
use toolbox_pty::strip_ansi;
use toolbox_state::diagnostics::{
    AnswerFault, Attempt, Collector, Failure, HelperFault, Known, Outcome, Program, Ran,
    Refusal as PlanRefusal, Source, outcome,
};

use crate::command::DEADLINE;
use crate::pkexec::{self, HelperError, HelperRun, Refusal};

/// English text in UTF-8 whatever the user's language: the report is read
/// by helpers in the forum.
const REPORT_LOCALE: &str = "C.UTF-8";

/// Collects diagnostic text. Blocks, so call it off the GUI thread.
pub trait DiagnosticSource: Send + Sync {
    /// What `collector` delivered, `None` when `cancel` was set meanwhile.
    fn collect(&self, collector: Collector, cancel: &Arc<AtomicBool>) -> Option<Outcome>;

    /// The machine's and the user's names and serials, which an upload
    /// replaces.
    fn known(&self) -> Known {
        Known::default()
    }
}

/// Starts one read of the root helper. The app knows where it lives and
/// whether pkexec starts it.
pub trait HelperReads: Send + Sync {
    fn read_command(&self, kind: ReadKind) -> Command;
}

pub struct SystemDiagnostics {
    root: PathBuf,
    /// Where the user's programs are looked up, `None` for `PATH`.
    programs: Option<PathBuf>,
    helper: Arc<dyn HelperReads>,
    /// `None` in tests, which know no names.
    known: Option<KnownRoots>,
}

impl SystemDiagnostics {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system(helper: Arc<dyn HelperReads>) -> Self {
        Self {
            root: PathBuf::from("/"),
            programs: None,
            helper,
            known: Some(KnownRoots::system()),
        }
    }

    /// Another root folder and program folder, for tests.
    #[must_use = "the adapter does nothing until asked"]
    #[cfg(test)]
    pub(crate) fn new(
        root: &std::path::Path,
        programs: &std::path::Path,
        helper: Arc<dyn HelperReads>,
    ) -> Self {
        Self {
            root: root.to_owned(),
            programs: Some(programs.to_owned()),
            helper,
            known: None,
        }
    }

    /// Runs the programs of an `a && b` chain until one does not succeed.
    fn run_chain(&self, programs: &[Program], cancel: &Arc<AtomicBool>) -> Option<Vec<Attempt>> {
        let mut attempts = Vec::new();

        for program in programs {
            let attempt = self.run(program, cancel)?;
            let go_on = matches!(&attempt, Attempt::Ran(ran) if ran.succeeded());

            attempts.push(attempt);

            if !go_on {
                break;
            }
        }

        Some(attempts)
    }

    fn run(&self, program: &Program, cancel: &Arc<AtomicBool>) -> Option<Attempt> {
        let mut command = self.programs.as_ref().map_or_else(
            || Command::new(locate_program(program.tool, Environment::Inherited)),
            |folder| Command::new(folder.join(program.tool.name())),
        );

        command
            .args(program.arguments)
            .env("LANG", REPORT_LOCALE)
            .env("LC_ALL", REPORT_LOCALE);

        let started = Instant::now();
        let result = capture(
            &mut command,
            Environment::Inherited,
            program.environment,
            &Policy::tail(MAX_READ_OUTPUT_BYTES).within(DEADLINE),
            cancel,
        );

        tracing::debug!(
            "{} {}: {} after {} ms",
            program.tool.name(),
            program.arguments.join(" "),
            describe(&result),
            started.elapsed().as_millis()
        );

        match result {
            Ok(captured) => Some(Attempt::Ran(ran(captured))),
            Err(CaptureError::Missing) => Some(Attempt::Missing),
            Err(CaptureError::Failed(error)) => Some(Attempt::Failed(Failure::Start {
                program: program.tool.name().to_owned(),
                error: error.to_string(),
            })),
            Err(CaptureError::TimedOut) => Some(timed_out(program)),
            Err(CaptureError::Cancelled) => None,
            Err(CaptureError::TooLarge) => Some(Attempt::Failed(Failure::TooLarge {
                program: program.tool.name().to_owned(),
            })),
        }
    }

    /// One read through the root helper. Its answer line holds what the
    /// program printed.
    fn ask_helper(&self, kind: ReadKind, cancel: &Arc<AtomicBool>) -> Option<Attempt> {
        let started = Instant::now();

        // No deadline: an unanswered polkit dialog waits for its user.
        let result = capture(
            &mut self.helper.read_command(kind),
            Environment::Inherited,
            &[],
            &Policy::tail(ProgramAnswer::MAX_LINE_BYTES),
            cancel,
        );

        tracing::debug!(
            "helper read {}: {} after {} ms",
            kind.as_str(),
            describe(&result),
            started.elapsed().as_millis()
        );

        match pkexec::sorted(result) {
            HelperRun::Answered(answer) => Some(answered(&answer)),
            HelperRun::Refused(refusal) => Some(refused(refusal)),
            HelperRun::Failed(error) => {
                Some(Attempt::Failed(Failure::Helper(helper_fault(&error))))
            }
            HelperRun::Cancelled => None,
        }
    }
}

impl DiagnosticSource for SystemDiagnostics {
    fn collect(&self, collector: Collector, cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        let attempts = match collector.source(access(&self.root)) {
            Source::Programs(programs) => self.run_chain(programs, cancel)?,
            Source::Helper(kind) => vec![self.ask_helper(kind, cancel)?],
            Source::PacmanLog => vec![pacman_log(&self.root, MAX_READ_OUTPUT_BYTES)],
        };

        // A failure's reason is shown as plain text, never as a report.
        Some(match outcome(&attempts) {
            Outcome::Failed(reason) => Outcome::Failed(reason.map_text(strip_ansi)),
            shown @ (Outcome::Report(_) | Outcome::Unavailable | Outcome::Refused(_)) => shown,
        })
    }

    fn known(&self) -> Known {
        self.known
            .as_ref()
            .map_or_else(Known::default, known::of_process)
    }
}

/// The helper's answer line: what the program printed, or that it is
/// not installed.
fn answered(answer: &str) -> Attempt {
    match ProgramAnswer::parse(answer.trim_end()) {
        Ok(ProgramAnswer::Ran {
            output,
            errors,
            code,
            truncated,
        }) => Attempt::Ran(ran(Captured {
            output,
            errors,
            code,
            truncated,
        })),
        Ok(ProgramAnswer::Missing) => Attempt::Missing,
        Err(error) => Attempt::Failed(Failure::Answer(answer_fault(&error))),
    }
}

/// A dialog that could never appear and a missing helper keep the
/// report's wording honest instead of blaming a closed dialog.
const fn refused(refusal: Refusal) -> Attempt {
    Attempt::Refused(match refusal {
        Refusal::Dismissed => PlanRefusal::Dismissed,
        Refusal::Denied => PlanRefusal::Denied,
        Refusal::NoAgent => PlanRefusal::NoAgent,
        Refusal::HelperMissing | Refusal::Broken => PlanRefusal::HelperMissing,
    })
}

/// The program's text with its colour codes, which the page shows. An
/// output of nothing but codes counts as none, so a failed program still
/// fails with its error text.
fn ran(captured: Captured) -> Ran {
    let blank =
        captured.output.contains('\u{1b}') && strip_ansi(&captured.output).trim().is_empty();

    Ran {
        output: if blank {
            String::new()
        } else {
            captured.output
        },
        errors: captured.errors,
        code: captured.code,
        truncated: captured.truncated,
    }
}

fn describe(result: &Result<Captured, CaptureError>) -> String {
    match result {
        Ok(captured) => captured.code.map_or_else(
            || "ended by a signal".to_owned(),
            |code| format!("exit code {code}"),
        ),
        Err(error) => error.to_string(),
    }
}

/// A collector's program ran past [`DEADLINE`]: it fails its collector
/// instead of the whole report.
fn timed_out(program: &Program) -> Attempt {
    Attempt::Failed(Failure::TimedOut {
        program: program.tool.name().to_owned(),
        seconds: DEADLINE.as_secs(),
    })
}

/// The report names why the helper did not start too, such as the
/// system's own words for a missing file.
fn helper_fault(error: &HelperError) -> HelperFault {
    match error {
        HelperError::Missing => HelperFault::Missing,
        HelperError::Start(source) => HelperFault::Start {
            error: source.to_string(),
        },
        HelperError::TimedOut => HelperFault::TimedOut,
        HelperError::TooLong => HelperFault::TooLong,
        HelperError::Pkexec { words } => HelperFault::Pkexec {
            words: words.clone(),
        },
        HelperError::Ended { code, words } => HelperFault::Ended {
            code: *code,
            words: words.clone(),
        },
        HelperError::Unreadable => HelperFault::Unreadable,
        HelperError::Stopped => HelperFault::Stopped,
    }
}

fn answer_fault(error: &DecodeError) -> AnswerFault {
    match error {
        DecodeError::TooLong { length, limit } => AnswerFault::TooLong {
            length: *length,
            limit: *limit,
        },
        DecodeError::Json(error) => AnswerFault::Json(error.to_string()),
    }
}
