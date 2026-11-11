//! A change as JSON Lines for `--json`, one object per line: `plan`, then
//! `started`, `output` and `finished` for each step, `error` for a problem
//! and `done` at the end. The names are part of the interface, as the
//! exit codes are. Each line starts with `"schema": 1`, the version of
//! these names.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::time::Instant;

use serde::Serialize;
use toolbox_cli::output::json_line;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_plan::{Plan, PlannedAction, StepId, UserAction};
use toolbox_services::apply::{Problem, Report};

use super::show::{Clock, failed, refused};

#[derive(Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
enum Line<'a> {
    Plan {
        steps: Vec<Planned<'a>>,
    },
    Started {
        step: usize,
        of: usize,
        command: &'a [String],
    },
    Output {
        step: usize,
        line: &'a str,
    },
    Finished {
        step: usize,
        outcome: Ended,
        seconds: f64,
    },
    Error {
        message: &'a str,
        /// The program's own words.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<&'a str>,
        /// What helps.
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<&'a str>,
    },
    Done {
        ok: bool,
    },
}

#[derive(Serialize)]
struct Planned<'a> {
    step: usize,
    command: &'a [String],
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Ended {
    Done,
    Failed,
    Cancelled,
    /// A dry run of the helper, or nothing to do for the step.
    Skipped,
}

pub struct Events {
    /// The plan's steps in their order, each with its command lines.
    steps: Vec<(StepId, Vec<String>)>,
    begun: HashMap<StepId, Instant>,
    clock: Clock,
    /// Output of the running step after its last line break.
    pending: String,
}

impl Events {
    pub fn new(clock: Clock) -> Self {
        Self {
            steps: Vec::new(),
            begun: HashMap::new(),
            clock,
            pending: String::new(),
        }
    }

    pub fn report(&mut self, report: &Report) -> String {
        match report {
            Report::PlanStarted { steps } => {
                self.steps.clone_from(steps);
                String::new()
            }
            Report::Step(Event::Started { step }) => {
                self.begun.insert(*step, (self.clock)());
                self.pending.clear();
                json_line(&Line::Started {
                    step: self.number(*step),
                    of: self.steps.len(),
                    command: self.command(*step),
                })
            }
            Report::Step(Event::Output { step, data }) => self.output(*step, data),
            Report::Step(Event::Finished { step, outcome }) => self.finished(*step, outcome),
            Report::Step(Event::Rejected { reason }) => {
                error("the helper refused the change", Some(reason))
            }
            Report::Step(Event::Accepted | Event::Done) | Report::CancelWaits => String::new(),
            Report::Problem(Problem::Refused(refusal)) => {
                let (said, hint) = refused(*refusal);
                error_with(&format!("not carried out: {said}"), Some(hint), None)
            }
            Report::Problem(Problem::HelperFailed { kind, detail }) => {
                error(failed(*kind), Some(detail))
            }
            Report::Busy => error("another change is still running", None),
            Report::PlanFinished { succeeded } => done(*succeeded),
        }
    }

    /// Where `step` stands in the plan, from 1.
    fn number(&self, step: StepId) -> usize {
        let index = self.steps.iter().position(|(id, _)| *id == step);
        index.map_or(0, |index| index + 1)
    }

    fn command(&self, step: StepId) -> &[String] {
        self.steps
            .iter()
            .find(|(id, _)| *id == step)
            .map_or(&[], |(_, command)| command.as_slice())
    }

    /// One `output` line for each line `data` completes. The rest waits
    /// for its line break, or for the step's end.
    fn output(&mut self, step: StepId, data: &str) -> String {
        self.pending.push_str(data);
        let Some(end) = self.pending.rfind('\n') else {
            return String::new();
        };

        let rest = self.pending.split_off(end + 1);
        let whole = std::mem::replace(&mut self.pending, rest);
        let number = self.number(step);

        whole
            .lines()
            .map(|line| json_line(&Line::Output { step: number, line }))
            .collect()
    }

    fn finished(&mut self, step: StepId, outcome: &Outcome) -> String {
        let number = self.number(step);
        let mut text = String::new();
        let rest = std::mem::take(&mut self.pending);
        if !rest.is_empty() {
            text.push_str(&json_line(&Line::Output {
                step: number,
                line: rest.trim_end_matches('\r'),
            }));
        }

        let seconds = self.begun.remove(&step).map_or(0.0_f64, |begun| {
            let took = (self.clock)().saturating_duration_since(begun);
            (took.as_secs_f64() * 1_000.0_f64).round() / 1_000.0_f64
        });

        let outcome = match outcome {
            Outcome::Succeeded => Ended::Done,
            Outcome::Failed {
                cause: _,
                cleanup_failed: _,
            } => Ended::Failed,
            Outcome::Cancelled => Ended::Cancelled,
            Outcome::Skipped => Ended::Skipped,
        };

        text.push_str(&json_line(&Line::Finished {
            step: number,
            outcome,
            seconds,
        }));
        text
    }
}

/// The `plan` line, the steps numbered from 1.
pub fn planned(plan: &Plan) -> String {
    let commands: Vec<Vec<String>> = plan
        .steps()
        .iter()
        .map(|step| step.action.command_lines())
        .collect();

    let steps = commands
        .iter()
        .enumerate()
        .map(|(index, command)| Planned {
            step: index + 1,
            command,
        })
        .collect();

    json_line(&Line::Plan { steps })
}

/// An `error` line. A second line of `text` is its hint, as the texts of
/// the commands carry it.
pub fn error(text: &str, detail: Option<&str>) -> String {
    let (message, hint) = text
        .split_once('\n')
        .map_or((text, None), |(message, hint)| (message, Some(hint)));

    error_with(message, hint, detail)
}

/// An `error` line with its hint and the program's own words apart.
pub fn error_with(message: &str, hint: Option<&str>, detail: Option<&str>) -> String {
    json_line(&Line::Error {
        message,
        detail: detail.filter(|detail| !detail.trim().is_empty()),
        hint,
    })
}

pub fn done(ok: bool) -> String {
    json_line(&Line::Done { ok })
}

/// What a change that would do nothing prints: a plan without steps that
/// ended well.
pub fn nothing() -> String {
    format!("{}{}", planned(&Plan::new([])), done(true))
}

/// A step that wants answers typed while it runs: paru's questions or an
/// editor. JSON Lines leave nobody to type them.
pub const fn needs_typing(action: &PlannedAction) -> bool {
    matches!(
        action,
        PlannedAction::User(
            UserAction::UpgradeAur
                | UserAction::InstallAur { packages: _ }
                | UserAction::EditFile { editor: _, copy: _ }
        )
    )
}
