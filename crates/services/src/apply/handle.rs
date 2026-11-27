//! The two ends of the apply service that the app holds: the handle the
//! window and the command line feed, and the reporter the plan's steps
//! write their events to.

use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::Event;

use super::gate::OutputGate;
use super::{ApplyCommand, Problem, Report};
use crate::waking::Waking;

/// The app's side of the service: commands in, reports out.
#[derive(Debug)]
pub struct ApplyHandle {
    pub(super) commands: UnboundedSender<ApplyCommand>,
    pub(super) reports: Receiver<Report>,
    pub(super) gate: OutputGate,
}

#[derive(Clone, Debug)]
pub(super) struct Reporter {
    pub(super) reports: Waking<Report>,
    pub(super) gate: OutputGate,
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

#[cfg(any(test, feature = "testing"))]
impl ApplyHandle {
    /// A handle without a service, fed through the sender.
    #[must_use = "the sender is the only way to feed the handle"]
    pub fn feeding(wake: crate::waking::Wake) -> (Waking<Report>, Self) {
        let (commands, _) = tokio::sync::mpsc::unbounded_channel();
        let (sender, reports) = crate::waking::channel(wake);
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
    pub(super) fn report(&self, report: Report) {
        self.reports.send(report);
    }

    /// A step's event, which waits while too much output is unread.
    pub(super) async fn step(&self, event: Event) {
        self.gate.enter(output_len(&event)).await;
        self.report(Report::Step(event));
    }

    /// [`Self::step`] for the blocking thread of the user steps.
    pub(super) fn step_blocking(&self, event: Event, runtime: &Handle) {
        self.gate.enter_blocking(output_len(&event), runtime);
        self.report(Report::Step(event));
    }

    pub(super) fn helper_failed(&self, kind: HelperFailure, detail: String) {
        tracing::error!("the plan stopped ({}): {detail}", kind.as_str());
        self.report(Report::Problem(Problem::HelperFailed { kind, detail }));
    }
}

const fn output_len(event: &Event) -> usize {
    match event {
        Event::Output { step: _, data } => data.len(),
        Event::Accepted
        | Event::Rejected { reason: _ }
        | Event::Waiting { step: _ }
        | Event::Started { step: _ }
        | Event::Finished {
            step: _,
            outcome: _,
        }
        | Event::Done => 0,
    }
}
