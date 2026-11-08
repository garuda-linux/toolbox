//! One user segment, run inside the app with the user's own environment.
//! The steps run on a blocking thread (the PTY runner is synchronous),
//! answers reach it through a channel.

use std::path::PathBuf;
use std::sync::mpsc;

use toolbox_core::features::console::HelperFailure;
use toolbox_exec::{Environment, EventSink, PtyRunner, apply};
use toolbox_plan::protocol::{Event, Mode};
use toolbox_plan::{Step, UserAction};

use super::{Report, Reporter, Run};

struct ReportSink(Reporter);

impl EventSink for ReportSink {
    fn send(&mut self, event: Event) {
        self.0.report(Report::Step(event));
    }
}

pub async fn run(steps: Vec<Step<UserAction>>, run: &mut Run<'_>, mode: Mode) -> bool {
    let (answers, received) = mpsc::channel();
    if let Some(size) = run.starting_size() {
        // The receiver is alive until the runner ends, below.
        answers.send(size).ok();
    }

    let mut sink = ReportSink(run.reporter.clone());

    // The blocking thread does not inherit the plan's span by itself.
    let plan = tracing::Span::current();
    let mut task = tokio::task::spawn_blocking(move || {
        let _plan = plan.entered();
        let runner = PtyRunner::new(received, Environment::Inherited);
        let mut runner = match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
            Some(home) => runner.with_home(PathBuf::from(home)),
            None => runner,
        };

        apply(mode, &steps, &mut runner, &mut sink)
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
