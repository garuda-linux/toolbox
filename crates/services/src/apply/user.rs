//! One user segment, run inside the app with the user's own environment.
//! The steps run on a blocking thread (the PTY runner is synchronous),
//! answers reach it through a channel.

use std::sync::{Arc, mpsc};

use tokio::runtime::Handle;
use toolbox_core::features::console::HelperFailure;
use toolbox_exec::EventSink;
use toolbox_plan::protocol::Event;
use toolbox_plan::{Step, UserAction};

use super::{Launcher, Reporter, Run};

struct ReportSink {
    reporter: Reporter,
    runtime: Handle,
}

impl EventSink for ReportSink {
    fn send(&mut self, event: Event) {
        self.reporter.step_blocking(event, &self.runtime);
    }
}

pub async fn run(
    steps: Vec<Step<UserAction>>,
    run: &mut Run<'_>,
    launcher: &Arc<dyn Launcher>,
) -> bool {
    let (answers, received) = mpsc::channel();
    if let Some(size) = run.starting_size() {
        // The receiver is alive until the runner ends, below.
        answers.send(size).ok();
    }

    let mut sink = ReportSink {
        reporter: run.reporter.clone(),
        runtime: Handle::current(),
    };

    // The blocking thread does not inherit the plan's span by itself.
    let plan = tracing::Span::current();
    let launcher = Arc::clone(launcher);
    let mut task = tokio::task::spawn_blocking(move || {
        let _plan = plan.entered();
        launcher.user_steps(&steps, received, &mut sink)
    });

    loop {
        tokio::select! {
            result = &mut task => {
                return result.unwrap_or_else(|error| {
                    run.reporter.helper_failed(HelperFailure::UserSteps, error.to_string());
                    false
                });
            }
            Some(command) = run.commands.recv() => {
                if let Some(answer) = run.answer_for(command) {
                    // The runner is gone only once the task ended, which the
                    // other branch reports.
                    answers.send(answer).ok();
                }
            }
        }
    }
}
