//! Carries out a plan.
//!
//! Root segments run through the helper, user segments in the app, one
//! after another. Keys typed into the Console, resizes and Cancel reach
//! whichever segment runs. Reports go to the app over a channel that
//! wakes the frame.

mod gate;
mod handle;
mod inhibit;
pub mod launcher;
mod root;
mod session;
#[cfg(test)]
mod tests;
mod user;

use std::sync::Arc;

use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Answer, Event, TerminalSize};
use toolbox_plan::{Plan, Segment, StepId};
use toolbox_system::pkexec::Refusal;
use tracing::Instrument as _;

use self::gate::OutputGate;
pub use self::handle::ApplyHandle;
use self::handle::Reporter;
pub use self::inhibit::Inhibitor;
pub use self::launcher::{HelperLauncher, Launcher};
use crate::waking::{self, Wake};

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

struct Run<'a> {
    commands: &'a mut UnboundedReceiver<ApplyCommand>,
    reporter: &'a Reporter,
    /// The Console's size, sent to every segment as it starts, since a
    /// new program would otherwise start at the pty's default size.
    size: &'a mut Option<TerminalSize>,
    /// A cancel or a finish came. The running step may still end well, but
    /// no segment behind it starts.
    stop_asked: bool,
}

impl Run<'_> {
    /// What a command means to the running segment. A second plan while
    /// one runs is refused, never queued. A resize is also kept for the
    /// next segment.
    fn answer_for(&mut self, command: ApplyCommand) -> Option<Answer> {
        match command {
            ApplyCommand::Input(bytes) => Some(Answer::Input {
                data: String::from_utf8_lossy(&bytes).into_owned(),
            }),
            ApplyCommand::Resize(size) => {
                *self.size = Some(size);
                Some(Answer::Resize { size })
            }
            ApplyCommand::Cancel => {
                self.stop_asked = true;
                Some(Answer::Cancel)
            }
            ApplyCommand::Finish => {
                self.stop_asked = true;
                Some(Answer::Finish)
            }
            ApplyCommand::Run { id, plan: _ } => {
                self.reporter.report(Report::Busy { plan: id });
                None
            }
        }
    }

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
                    stop_asked: false,
                };

                // Only the number of steps: a plan may hold whole config
                // files.
                let span = tracing::info_span!("apply", steps = plan.steps().len());
                let hold = launcher.inhibitor().hold(launcher.mode()).await;
                let succeeded = run_plan(id, plan, &mut run, &launcher)
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

async fn run_plan(id: PlanId, plan: Plan, run: &mut Run<'_>, launcher: &Arc<dyn Launcher>) -> bool {
    tracing::info!("the plan started");
    let listed = plan
        .steps()
        .iter()
        .map(|step| (step.id, step.action.commands()))
        .collect();

    run.reporter.report(Report::PlanStarted {
        plan: id,
        steps: listed,
    });

    let segments = plan.into_segments();
    let request = root::request(&segments, launcher.mode());
    let mut session = None;
    let mut succeeded = true;

    let mut rest = segments.into_iter().peekable();
    while let Some(segment) = rest.next() {
        succeeded = match segment {
            Segment::Root(steps) => {
                root::run(&steps, &request, &mut session, run, launcher.as_ref()).await
            }
            Segment::User(steps) => user::run(steps, run, launcher).await,
        };

        // A step that ended well after a cancel or a finish is still the
        // last one: the helper gets its cancel at the gate when it closes.
        if run.stop_asked && rest.peek().is_some() {
            succeeded = false;
        }

        if !succeeded {
            tracing::info!("the plan stopped early");
            break;
        }
    }

    if let Some(open) = session {
        succeeded &= open.close(!succeeded, run, launcher.as_ref()).await;
    }

    if succeeded {
        tracing::info!("the plan finished");
    }

    succeeded
}
