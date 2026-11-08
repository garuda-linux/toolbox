//! The Diagnostics page's collectors against the running system: programs
//! as the user, fixed reads of the root helper where the user's rights do
//! not reach, and pacman's log.

mod access;
pub mod page;
pub mod service;
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
use toolbox_exec::{CaptureError, Captured, Environment, Policy, capture};
use toolbox_plan::protocol::{MAX_READ_OUTPUT_BYTES, ProgramAnswer, ReadKind};
use toolbox_pty::strip_ansi;
use toolbox_state::diagnostics::{
    Attempt, Collector, Outcome, Program, Ran, Refusal as PlanRefusal, Source, outcome,
};

use crate::command::DEADLINE;
use crate::pkexec::{self, Refusal};

/// English text in UTF-8 whatever the user's language: the report is read
/// by helpers in the forum.
const REPORT_LOCALE: &str = "C.UTF-8";

/// Collects diagnostic text. Blocks, so call it off the GUI thread.
pub trait DiagnosticSource: Send + Sync {
    /// What `collector` delivered, `None` when `cancel` was set meanwhile.
    fn collect(&self, collector: Collector, cancel: &Arc<AtomicBool>) -> Option<Outcome>;
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
}

impl SystemDiagnostics {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system(helper: Arc<dyn HelperReads>) -> Self {
        Self {
            root: PathBuf::from("/"),
            programs: None,
            helper,
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
            || Command::new(program.name),
            |folder| Command::new(folder.join(program.name)),
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
            program.name,
            program.arguments.join(" "),
            describe(&result),
            started.elapsed().as_millis()
        );

        match result {
            Ok(captured) => Some(Attempt::Ran(ran(captured))),
            Err(CaptureError::Missing) => Some(Attempt::Missing),
            Err(CaptureError::Failed(error)) => Some(Attempt::Failed(format!(
                "{} could not be started: {error}",
                program.name
            ))),
            Err(CaptureError::TimedOut) => Some(timed_out(program)),
            Err(CaptureError::Cancelled) => None,
            Err(CaptureError::TooLarge) => Some(Attempt::Failed(format!(
                "{} printed too much",
                program.name
            ))),
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

        match result {
            Ok(captured) => Some(helper_attempt(&captured)),
            Err(CaptureError::Missing) => Some(Attempt::Failed(
                "The helper of Garuda Toolbox or pkexec is not installed.".to_owned(),
            )),
            Err(CaptureError::Failed(error)) => Some(Attempt::Failed(format!(
                "The helper of Garuda Toolbox could not be started: {error}"
            ))),
            // Without a deadline and with a tail policy neither can happen.
            Err(CaptureError::TimedOut | CaptureError::TooLarge) => Some(Attempt::Failed(
                "The helper of Garuda Toolbox gave no answer".to_owned(),
            )),
            Err(CaptureError::Cancelled) => None,
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
            Outcome::Failed(reason) => Outcome::Failed(strip_ansi(&reason)),
            shown @ (Outcome::Report(_) | Outcome::Unavailable | Outcome::Refused(_)) => shown,
        })
    }
}

/// What the helper's run means: refused by pkexec, an answer line, or a
/// failure with the helper's own words.
fn helper_attempt(captured: &Captured) -> Attempt {
    match pkexec::refusal(captured.code, &captured.errors) {
        Some(Refusal::Broken) => return Attempt::Failed(pkexec::broken(&captured.errors)),
        Some(refusal) => return refused(refusal),
        None => {}
    }

    match captured.code {
        Some(0) => match ProgramAnswer::parse(captured.output.trim_end()) {
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
            Err(error) => Attempt::Failed(format!(
                "The answer of the helper of Garuda Toolbox was not understood: {error}"
            )),
        },
        Some(_) | None => Attempt::Failed(format!(
            "The helper of Garuda Toolbox failed: {}",
            strip_ansi(captured.errors.trim())
        )),
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
    Attempt::Failed(format!(
        "{} took longer than {} seconds",
        program.name,
        DEADLINE.as_secs()
    ))
}
