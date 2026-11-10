//! How a running change reads in the terminal: a `Running` section with
//! each step, the program's output under it and a marked line for how the
//! step ended, problems as `error:` lines, and one closing line.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::time::Instant;

use toolbox_cli::output::{INDENT, Mark, Paint, Row, Style};
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_services::apply::{Problem, Report};
use toolbox_system::pkexec::Refusal;

use super::elapsed::elapsed;
use crate::out::failure_text;

/// Where the time of a step comes from, so the tests can set it.
pub type Clock = Box<dyn Fn() -> Instant>;

/// A failed step's program words: a hint how to see them, or with
/// `--verbose` the words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Words {
    Hinted,
    Shown,
}

pub struct Show {
    paint: Paint,
    words: Words,
    /// Lines that wait for their mark start this far in, so their text
    /// lines up with the marked ones.
    room: &'static str,
    closing: &'static str,
    steps: Vec<(StepId, String)>,
    begun: HashMap<StepId, Instant>,
    started: Option<Instant>,
    clock: Clock,
    /// Something of the run is out, so the next section is set apart.
    printed: bool,
    /// The `Running` title is out.
    running: bool,
    /// The last output ended its line, so the next one starts indented.
    at_start: bool,
    /// The step that was cancelled, so a run that did not finish was
    /// stopped.
    cancelled: Option<StepId>,
}

impl Show {
    pub fn new(style: Style, dry_run: bool, clock: Clock) -> Self {
        Self {
            paint: Paint::new(style),
            words: Words::Hinted,
            room: if style == Style::Color { "  " } else { "" },
            closing: if dry_run {
                "Test mode, nothing was changed"
            } else {
                "Done"
            },
            steps: Vec::new(),
            begun: HashMap::new(),
            started: None,
            clock,
            printed: false,
            running: false,
            at_start: true,
            cancelled: None,
        }
    }

    #[must_use = "the verbose show is only of use once it reports"]
    pub const fn verbose(mut self, verbose: bool) -> Self {
        self.words = if verbose { Words::Shown } else { Words::Hinted };
        self
    }

    /// The text `report` adds, maybe none.
    pub fn report(&mut self, report: &Report) -> String {
        match report {
            Report::PlanStarted { steps } => {
                self.steps.clone_from(steps);
                self.started = Some((self.clock)());
                String::new()
            }
            Report::Step(Event::Started { step }) => {
                self.begun.insert(*step, (self.clock)());
                let counter = self.paint.dim(&self.counter(*step));
                let command = format!("{INDENT}{}{counter} {}", self.room, self.command(*step));
                self.step(&command)
            }
            Report::Step(Event::Output { step: _, data }) => self.output(data),
            Report::Step(Event::Finished { step, outcome }) => self.finished(*step, outcome),
            Report::Step(Event::Rejected { reason }) => {
                let text = self
                    .paint
                    .error(&format!("the helper refused the change: {reason}"));

                self.line(&text)
            }
            Report::Step(Event::Accepted | Event::Done) | Report::CancelWaits => String::new(),
            Report::Problem(problem) => self.problem(problem),
            Report::Busy => {
                let text = self.paint.error("another change is still running");
                self.line(&text)
            }
            Report::PlanFinished { succeeded } => self.result(*succeeded),
        }
    }

    fn command(&self, step: StepId) -> &str {
        self.steps
            .iter()
            .find(|(id, _)| *id == step)
            .map_or("", |(_, command)| command.as_str())
    }

    /// `[2/5]`, where `step` stands in the plan.
    fn counter(&self, step: StepId) -> String {
        let number = self.steps.iter().position(|(id, _)| *id == step);
        let number = number.map_or(0, |index| index + 1);
        format!("[{number}/{}]", self.steps.len())
    }

    fn output(&mut self, data: &str) -> String {
        let mut text = self.title_once();
        for piece in data.split_inclusive('\n') {
            if self.at_start {
                text.push_str(INDENT);
                text.push_str(self.room);
                text.push_str(INDENT);
            }

            text.push_str(piece);
            self.at_start = piece.ends_with('\n');
        }

        text
    }

    fn finished(&mut self, step: StepId, outcome: &Outcome) -> String {
        let command = self.command(step).to_owned();
        let (mark, text) = match outcome {
            Outcome::Skipped => return String::new(),
            Outcome::Succeeded => (Mark::On, format!("{command} done")),
            Outcome::Failed {
                cause: _,
                cleanup_failed: _,
            } => (Mark::Problem, format!("{command} failed")),
            Outcome::Cancelled => {
                self.cancelled = Some(step);
                (Mark::Off, format!("{command} was cancelled"))
            }
        };

        let mut line = marked(self.paint, mark, &text);
        if let Some(begun) = self.begun.remove(&step) {
            let took = (self.clock)().saturating_duration_since(begun);
            line.push(' ');
            line.push_str(&self.paint.dim(&format!("({})", elapsed(took))));
        }

        self.step(&line)
    }

    fn problem(&mut self, problem: &Problem) -> String {
        let text = match problem {
            Problem::Refused(refusal) => {
                let (said, hint) = refused(*refusal);
                let error = self.paint.error(&format!("not carried out: {said}"));
                format!("{error}\n{}", self.paint.hint(hint))
            }
            Problem::HelperFailed { kind, detail } => {
                let text = failure_text(
                    self.paint,
                    failed(*kind),
                    detail,
                    self.words == Words::Shown,
                );

                text.trim_end_matches('\n').to_owned()
            }
        };

        self.line(&text)
    }

    fn result(&mut self, succeeded: bool) -> String {
        let total = self
            .started
            .take()
            .map(|start| elapsed((self.clock)().saturating_duration_since(start)));

        let text = if succeeded {
            self.paint.title(self.closing, total.as_deref())
        } else if self.cancelled.is_some() {
            self.paint.title("Cancelled", total.as_deref())
        } else {
            self.paint.error("the change did not finish")
        };

        let apart = if self.printed { "\n" } else { "" };
        let text = format!("{apart}{text}");
        self.line(&text)
    }

    pub fn note(&mut self, text: &str) -> String {
        let text = self.paint.warning(text);
        self.line(&text)
    }

    /// `Running`, the first time a step shows.
    fn title_once(&mut self) -> String {
        if self.running {
            return String::new();
        }

        self.running = true;
        let apart = self.apart();
        format!("{apart}{}\n\n", self.paint.title("Running", None))
    }

    fn step(&mut self, text: &str) -> String {
        let title = self.title_once();
        format!("{title}{}", self.close(text))
    }

    /// `text` on a line of its own, set apart from what came before the run.
    fn line(&mut self, text: &str) -> String {
        let apart = self.apart();
        format!("{apart}{}", self.close(text))
    }

    /// One blank line before the first thing the run prints, so it stands
    /// apart from the plan above it.
    const fn apart(&mut self) -> &'static str {
        if self.printed {
            return "";
        }

        self.printed = true;
        "\n"
    }

    /// `text` on a line of its own, after any output that ended mid-line.
    fn close(&mut self, text: &str) -> String {
        let start = if self.at_start { "" } else { "\n" };
        self.at_start = true;
        format!("{start}{text}\n")
    }
}

/// `text` behind `mark` as a list draws it, the mark only on a terminal.
fn marked(paint: Paint, mark: Mark, text: &str) -> String {
    // A list of one row keeps the look of every list, without a copy of
    // its colours here.
    let list = paint.list("", None, &[], &[Row::new(mark, vec![text.to_owned()])]);
    list.lines().last().unwrap_or_default().to_owned()
}

/// What a failed helper means. Its own words show only with `--verbose`.
pub(super) const fn failed(kind: HelperFailure) -> &'static str {
    match kind {
        HelperFailure::MissingSecret => "a step needs a password that was not given",
        HelperFailure::Start => "the helper did not start",
        HelperFailure::Pkexec => "pkexec cannot run on this system",
        HelperFailure::NoPipes => "the helper could not be talked to",
        HelperFailure::Lost | HelperFailure::Ended => "the helper ended before the change was done",
        HelperFailure::UserSteps => "the steps run as you stopped",
    }
}

/// What pkexec's refusal means, and what helps.
pub(super) const fn refused(refusal: Refusal) -> (&'static str, &'static str) {
    match refusal {
        Refusal::Dismissed => (
            "the password dialog was closed",
            "run it again and enter your password",
        ),
        Refusal::Denied => (
            "the password dialog was closed, the password was wrong, or you are no administrator",
            "an administrator is a member of the group wheel",
        ),
        Refusal::NoAgent => (
            "no password dialog could appear",
            "run it in a terminal, or in a desktop session with a polkit agent",
        ),
        Refusal::HelperMissing => ("the helper is not installed", "reinstall garuda-toolbox"),
        Refusal::Broken => (
            "pkexec cannot run on this system",
            "check that pkexec is installed and setuid root",
        ),
    }
}
