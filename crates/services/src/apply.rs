//! Carries out a plan.
//!
//! Root segments run through the helper, user segments in the app, one
//! after another. Keys typed into the Console, resizes and Cancel reach
//! whichever segment runs. Reports go to the app over a channel that
//! wakes the frame.

mod gate;
mod inhibit;
pub mod launcher;
mod root;
#[cfg(test)]
mod tests;
mod user;

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Answer, Event, Outcome, TerminalSize};
use toolbox_plan::{Plan, Segment, StepId};
use toolbox_system::pkexec::Refusal;
use tracing::Instrument as _;

use self::gate::OutputGate;
pub use self::inhibit::Inhibitor;
pub use self::launcher::{HelperLauncher, Launcher};
use crate::waking::{self, Wake, Waking};

#[derive(Debug)]
pub enum ApplyCommand {
    /// `id` comes back in the plan's reports, so its sender tells them
    /// from those of another plan.
    Run {
        id: PlanId,
        plan: Plan,
    },
    /// Bytes for the running program, as the terminal encodes keys.
    Input(Vec<u8>),
    Resize(TerminalSize),
    Cancel,
    /// Lets the running step end and skips the rest: the command line's
    /// first Ctrl+C.
    Finish,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    /// Every step with its command lines, for the step list and so a dry
    /// run's success can be checked against what was really sent.
    PlanStarted {
        plan: PlanId,
        steps: Vec<(StepId, Vec<String>)>,
    },
    /// A step's progress or output, from the helper or the app.
    Step(Event),
    /// A problem outside any step: pkexec refused, the helper died, …
    Problem(Problem),
    PlanFinished {
        plan: PlanId,
        succeeded: bool,
    },
    /// Another plan runs already. The new one, `plan`, is refused, never
    /// queued.
    Busy {
        plan: PlanId,
    },
    /// Cancel came while pkexec still asks for the password. pkexec is
    /// left alone, so the cancel takes effect once the dialog is answered.
    CancelWaits,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// pkexec did not run the helper.
    Refused(Refusal),
    /// The helper or the user steps failed outside a step. `detail` holds
    /// the system's words, or the step number for
    /// [`HelperFailure::MissingSecret`], which has none.
    HelperFailed { kind: HelperFailure, detail: String },
}

/// Names a sent plan. Its sender counts them, so two plans sent within
/// one frame still tell their reports apart.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanId(u64);

/// The app's side of the service: commands in, reports out.
#[derive(Debug)]
pub struct ApplyHandle {
    commands: UnboundedSender<ApplyCommand>,
    reports: Receiver<Report>,
    gate: OutputGate,
}

#[derive(Clone, Debug)]
struct Reporter {
    reports: Waking<Report>,
    gate: OutputGate,
}

impl Report {
    pub const fn output_len(&self) -> usize {
        match self {
            Self::Step(event) => output_len(event),
            Self::PlanStarted { plan: _, steps: _ }
            | Self::Problem(_)
            | Self::PlanFinished {
                plan: _,
                succeeded: _,
            }
            | Self::Busy { plan: _ }
            | Self::CancelWaits => 0,
        }
    }
}

impl PlanId {
    pub const fn new(number: u64) -> Self {
        Self(number)
    }

    /// The id of the plan its sender sends after this one.
    #[must_use = "returns the next id, this one stays as it was"]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[cfg(any(test, feature = "testing"))]
impl ApplyHandle {
    /// A handle without a service, fed through the sender.
    #[must_use = "the sender is the only way to feed the handle"]
    pub fn feeding(wake: Wake) -> (Waking<Report>, Self) {
        let (commands, _) = unbounded_channel();
        let (sender, reports) = waking::channel(wake);
        let handle = Self {
            commands,
            reports,
            gate: OutputGate::default(),
        };

        (sender, handle)
    }
}

impl ApplyHandle {
    /// Sending fails only after shutdown, when nobody listens anymore.
    pub fn send(&self, command: ApplyCommand) {
        if self.commands.send(command).is_err() {
            tracing::debug!("the apply service is gone");
        }
    }

    /// Reports in order until they carry `budget` bytes of program output,
    /// and whether more wait. The window feeds output to its terminal
    /// within the frame, so a flood is spread over frames instead of one
    /// that never ends.
    pub fn reports_within(&self, budget: usize) -> (Vec<Report>, bool) {
        let mut taken = Vec::new();
        let mut spent = 0;
        while spent < budget {
            let Ok(report) = self.reports.try_recv() else {
                return (taken, false);
            };

            spent += report.output_len();
            taken.push(report);
        }

        self.gate.leave(spent);
        (taken, !self.reports.is_empty())
    }

    /// The next report, `None` once the service is gone. For the command
    /// line, which has no frame to wait for.
    pub fn wait(&self) -> Option<Report> {
        let report = self.reports.recv().ok()?;
        self.gate.leave(report.output_len());
        Some(report)
    }

    /// The next report within `wait`, for a command line that also looks
    /// for Ctrl+C in between.
    ///
    /// # Errors
    ///
    /// When `wait` passed without one, or the service is gone.
    pub fn wait_for(&self, wait: Duration) -> Result<Report, RecvTimeoutError> {
        let report = self.reports.recv_timeout(wait)?;
        self.gate.leave(report.output_len());
        Ok(report)
    }
}

impl Reporter {
    fn report(&self, report: Report) {
        self.reports.send(report);
    }

    /// A step's event, which waits while too much output is unread.
    async fn step(&self, event: Event) {
        self.gate.enter(output_len(&event)).await;
        self.report(Report::Step(event));
    }

    /// [`Self::step`] for the blocking thread of the user steps.
    fn step_blocking(&self, event: Event, runtime: &Handle) {
        self.gate.enter_blocking(output_len(&event), runtime);
        self.report(Report::Step(event));
    }

    /// Logs the failure once, where it happens, and tells the Console.
    fn helper_failed(&self, kind: HelperFailure, detail: String) {
        tracing::error!("the plan stopped ({}): {detail}", kind.as_str());
        self.report(Report::Problem(Problem::HelperFailed { kind, detail }));
    }
}

struct Run<'a> {
    commands: &'a mut UnboundedReceiver<ApplyCommand>,
    reporter: &'a Reporter,
    /// The Console's size, sent to every segment as it starts, since a
    /// new program would otherwise start at the pty's default size.
    size: &'a mut Option<TerminalSize>,
}

impl Run<'_> {
    /// What a command means to the running segment. A second plan while
    /// one runs is refused, never queued: the user starts it again
    /// knowingly. A resize is also kept for the next segment.
    fn answer_for(&mut self, command: ApplyCommand) -> Option<Answer> {
        match command {
            ApplyCommand::Input(bytes) => Some(Answer::Input {
                data: String::from_utf8_lossy(&bytes).into_owned(),
            }),
            ApplyCommand::Resize(size) => {
                *self.size = Some(size);
                Some(Answer::Resize { size })
            }
            ApplyCommand::Cancel => Some(Answer::Cancel),
            ApplyCommand::Finish => Some(Answer::Finish),
            ApplyCommand::Run { id, plan: _ } => {
                self.reporter.report(Report::Busy { plan: id });
                None
            }
        }
    }

    /// The size answer a new segment starts with, if the Console has one.
    fn starting_size(&self) -> Option<Answer> {
        self.size.map(|size| Answer::Resize { size })
    }
}

#[must_use = "the handle is the only way to the service"]
pub fn spawn(runtime: &Handle, wake: Wake, launcher: Arc<dyn Launcher>) -> ApplyHandle {
    let (commands, command_queue) = unbounded_channel();
    let (reports, report_queue) = waking::channel(wake);
    let gate = OutputGate::default();
    let reporter = Reporter {
        reports,
        gate: gate.clone(),
    };

    drop(runtime.spawn(serve(command_queue, launcher, reporter)));
    ApplyHandle {
        commands,
        reports: report_queue,
        gate,
    }
}

async fn serve(
    mut commands: UnboundedReceiver<ApplyCommand>,
    launcher: Arc<dyn Launcher>,
    reporter: Reporter,
) {
    let mut size = None;

    while let Some(command) = commands.recv().await {
        match command {
            ApplyCommand::Run { id, plan } => {
                let mut run = Run {
                    commands: &mut commands,
                    reporter: &reporter,
                    size: &mut size,
                };

                // Only the number of steps: a plan may hold whole config
                // files.
                let span = tracing::info_span!("apply", steps = plan.steps().len());
                let hold = launcher.inhibitor().hold(launcher.mode()).await;
                let succeeded = run_plan(id, plan, &mut run, launcher.as_ref())
                    .instrument(span)
                    .await;

                drop(hold);
                reporter.report(Report::PlanFinished {
                    plan: id,
                    succeeded,
                });
            }
            ApplyCommand::Resize(new) => size = Some(new),
            ApplyCommand::Input(_) | ApplyCommand::Cancel | ApplyCommand::Finish => {
                tracing::debug!("no plan runs, so input, cancel and finish do nothing");
            }
        }
    }
}

async fn run_plan(id: PlanId, plan: Plan, run: &mut Run<'_>, launcher: &dyn Launcher) -> bool {
    tracing::info!("the plan started");
    let listed = plan
        .steps()
        .iter()
        .map(|step| (step.id, step.action.command_lines()))
        .collect();

    run.reporter.report(Report::PlanStarted {
        plan: id,
        steps: listed,
    });

    for segment in plan.into_segments() {
        let succeeded = match segment {
            Segment::Root(steps) => root::run(&steps, run, launcher).await,
            Segment::User(steps) => user::run(steps, run, launcher.mode()).await,
        };

        if !succeeded {
            tracing::info!("the plan stopped early");
            return false;
        }
    }

    tracing::info!("the plan finished");
    true
}

const fn step_ok(event: &Event) -> Option<bool> {
    match event {
        Event::Finished { step: _, outcome } => {
            Some(matches!(outcome, Outcome::Succeeded | Outcome::Skipped))
        }
        Event::Rejected { reason: _ } => Some(false),
        Event::Accepted
        | Event::Started { step: _ }
        | Event::Output { step: _, data: _ }
        | Event::Done => None,
    }
}

const fn output_len(event: &Event) -> usize {
    match event {
        Event::Output { step: _, data } => data.len(),
        Event::Accepted
        | Event::Rejected { reason: _ }
        | Event::Started { step: _ }
        | Event::Finished {
            step: _,
            outcome: _,
        }
        | Event::Done => 0,
    }
}
