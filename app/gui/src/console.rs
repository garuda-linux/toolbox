mod mirror;
#[cfg(test)]
mod tests;

use mirror::MirrorWatch;
use toolbox_core::features::console::ending::Ending;
use toolbox_core::features::diagnostics::{Known, Redacted, redact};
use toolbox_plan::protocol::{self, Failure, Mode};
use toolbox_plan::{PackageName, Plan, StepId, TargetList};
use toolbox_system::share::Upload;
use toolbox_terminal::{Terminal, TerminalSize};

use crate::apply::{Problem, Report};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    Waiting,
    Running,
    Succeeded,
    Skipped,
    Cancelled,
    Failed(Failure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepRow {
    pub id: StepId,
    pub command: String,
    pub state: StepState,
}

pub struct Console {
    terminal: Terminal,
    steps: Vec<StepRow>,
    status: Status,
    problem: Option<Problem>,
    rejected: Option<String>,
    /// A plan was refused because another one runs.
    refused: bool,
    cancel_waits: bool,
    output: bool,
    share: Upload,
    mirror: MirrorWatch,
    /// Plans started so far, so each one's end shows once as new.
    runs: u64,
    /// What a shared output must not name.
    known: Known,
}

impl Console {
    pub fn new() -> Self {
        Self {
            terminal: Terminal::new(TerminalSize {
                columns: 100,
                rows: 30,
            }),
            steps: Vec::new(),
            status: Status::Idle,
            problem: None,
            rejected: None,
            refused: false,
            cancel_waits: false,
            runs: 0,
            output: false,
            share: Upload::default(),
            mirror: MirrorWatch::default(),
            known: Known::default(),
        }
    }

    pub fn knowing(known: Known) -> Self {
        Self {
            known,
            ..Self::new()
        }
    }

    /// The number of the plan shown, counting from 1. 0 before any.
    pub const fn run(&self) -> u64 {
        self.runs
    }

    pub fn take(&mut self, report: Report) {
        match report {
            Report::PlanStarted { steps } => {
                self.steps = steps
                    .into_iter()
                    .map(|(id, command)| StepRow {
                        id,
                        command: without_comment(&command).to_owned(),
                        state: StepState::Waiting,
                    })
                    .collect();
                self.status = Status::Running;
                self.runs += 1;
                self.problem = None;
                self.rejected = None;
                self.refused = false;
                self.cancel_waits = false;
                self.mirror.started();
            }
            Report::Step(event) => {
                self.cancel_waits = false;
                self.take_event(event);
            }
            Report::Problem(problem) => {
                self.cancel_waits = false;
                self.problem = Some(problem);
            }
            Report::PlanFinished { succeeded } => {
                self.refused = false;
                self.status = if succeeded {
                    Status::Succeeded
                } else {
                    Status::Failed
                };
            }
            // A notice beside the run: the running plan keeps its status.
            Report::Busy => {
                self.refused = true;
                self.mirror.refused();
            }
            Report::CancelWaits => self.cancel_waits = true,
        }
    }

    /// Returns the size to tell the program when it changed, so the program and
    /// the drawn grid always agree.
    pub fn resize(&mut self, fitting: TerminalSize) -> Option<protocol::TerminalSize> {
        let size = toolbox_pty::clamped(fitting);

        if size == self.terminal.size() {
            return None;
        }

        self.terminal.resize(size);
        Some((size.columns, size.rows).into())
    }

    fn take_event(&mut self, event: protocol::Event) {
        let (step, state) = match event {
            protocol::Event::Started { step } => (step, StepState::Running),
            protocol::Event::Finished { step, outcome } => {
                let failed = matches!(
                    outcome,
                    protocol::Outcome::Failed {
                        cause: _,
                        cleanup_failed: _
                    }
                );

                self.mirror.finished(step, !failed);
                (step, finished(&outcome))
            }
            protocol::Event::Output { step, data } => {
                self.output |= !data.is_empty();
                self.mirror.output(step, &data);
                self.terminal.feed_str(&data);
                return;
            }
            protocol::Event::Rejected { reason } => {
                self.rejected = Some(reason);
                return;
            }
            protocol::Event::Accepted | protocol::Event::Done => return,
        };

        if let Some(row) = self.steps.iter_mut().find(|row| row.id == step) {
            row.state = state;
        }
    }

    /// `plan` goes to the apply service, so its install steps are known
    /// once it starts.
    pub fn plan_sent(&mut self, plan: &Plan) {
        self.mirror.sent(plan);
    }

    /// The packages of a failed install whose mirror lacked a file, for
    /// the offer to update the system first and try again.
    pub const fn mirror_retry(&self) -> Option<&TargetList<PackageName>> {
        self.mirror.offer()
    }

    pub const fn terminal_mut(&mut self) -> &mut Terminal {
        &mut self.terminal
    }

    pub fn steps(&self) -> &[StepRow] {
        &self.steps
    }

    pub const fn status(&self) -> Status {
        self.status
    }

    pub const fn problem(&self) -> Option<&Problem> {
        self.problem.as_ref()
    }

    /// The helper's reason for refusing the plan, as it wrote it.
    pub fn rejected(&self) -> Option<&str> {
        self.rejected.as_deref()
    }

    pub fn ending(&self, mode: Mode) -> Ending {
        let count = |wanted: StepState| self.steps.iter().filter(|row| row.state == wanted).count();
        Ending::of(mode, count(StepState::Succeeded), count(StepState::Skipped))
    }

    pub const fn has_output(&self) -> bool {
        self.output
    }

    pub fn output_text(&self) -> String {
        self.terminal.to_plain_text()
    }

    /// The output as it may leave the machine.
    pub fn shareable_output(&self) -> Redacted {
        redact(&self.output_text(), &self.known)
    }

    /// Returns whether it did: not while a plan runs, whose programs still write to it.
    /// The last upload's link stays until it is hidden.
    pub fn clear_output(&mut self) -> bool {
        if self.status == Status::Running {
            return false;
        }

        self.terminal = Terminal::new(self.terminal.size());
        self.output = false;
        self.steps.clear();
        self.status = Status::Idle;
        self.problem = None;
        self.rejected = None;
        self.refused = false;
        self.cancel_waits = false;
        self.mirror.clear();

        // A running upload keeps its row, so its link still has a place
        // to arrive.
        if !self.share.running() {
            self.share.clear();
        }

        true
    }

    pub const fn share(&self) -> &Upload {
        &self.share
    }

    pub const fn share_mut(&mut self) -> &mut Upload {
        &mut self.share
    }

    /// Buttons that would start a plan wait while this holds.
    pub fn running(&self) -> bool {
        self.status == Status::Running
    }

    pub const fn refused(&self) -> bool {
        self.refused
    }

    pub const fn cancel_waits(&self) -> bool {
        self.cancel_waits
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

const fn finished(outcome: &protocol::Outcome) -> StepState {
    match outcome {
        protocol::Outcome::Succeeded => StepState::Succeeded,
        protocol::Outcome::Skipped => StepState::Skipped,
        protocol::Outcome::Cancelled => StepState::Cancelled,
        protocol::Outcome::Failed {
            cause,
            cleanup_failed: _,
        } => StepState::Failed(*cause),
    }
}

/// `command` without a trailing shell comment. Plans note why a step runs
/// after `#`, which is for the log, not for the step's line.
fn without_comment(command: &str) -> &str {
    let mut quote = None;
    let mut previous = ' ';
    for (at, character) in command.char_indices() {
        match (quote, character) {
            (None, '\'' | '"') => quote = Some(character),
            (Some(open), _) if open == character => quote = None,
            (None, '#') if previous.is_whitespace() => {
                return command.get(..at).map_or(command, str::trim_end);
            }
            _ => {}
        }

        previous = character;
    }

    command
}
