use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::unbounded_channel;
use toolbox_plan::protocol::Mode;

use super::{FakeLauncher, recording_script, root_plan};
use crate::apply::{ApplyCommand, OutputGate, PlanId, Report, Reporter, serve};

/// The command line's first Ctrl+C reaches the helper as its own answer,
/// so the running step can end on its own.
#[tokio::test]
async fn a_finish_during_a_run_reaches_the_helper() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = FakeLauncher {
        script: recording_script(&record, 2),
        mode: Mode::Run,
    };

    serve_all(
        launcher,
        vec![
            ApplyCommand::Run {
                id: PlanId::new(1),
                plan: root_plan(),
            },
            ApplyCommand::Finish,
        ],
    )
    .await;

    let sent = std::fs::read_to_string(&record).unwrap();

    assert_eq!(sent.lines().nth(1), Some("{\"type\":\"finish\"}"));
}

/// The helper waits for the Finish after the second plan, so that plan
/// arrives while the first runs.
#[tokio::test]
async fn a_plan_sent_during_a_run_is_refused_under_its_own_id() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = FakeLauncher {
        script: recording_script(&dir.path().join("stdin"), 2),
        mode: Mode::Run,
    };

    let reports = serve_all(
        launcher,
        vec![
            ApplyCommand::Run {
                id: PlanId::new(1),
                plan: root_plan(),
            },
            ApplyCommand::Run {
                id: PlanId::new(2),
                plan: root_plan(),
            },
            ApplyCommand::Finish,
        ],
    )
    .await;

    let ids: Vec<_> = reports
        .iter()
        .filter_map(|report| match report {
            Report::PlanStarted { plan, steps: _ } => Some(("started", *plan)),
            Report::Busy { plan } => Some(("refused", *plan)),
            Report::PlanFinished { plan, succeeded: _ } => Some(("finished", *plan)),
            Report::Step(_) | Report::Problem(_) | Report::CancelWaits => None,
        })
        .collect();

    assert_eq!(
        ids,
        [
            ("started", PlanId::new(1)),
            ("refused", PlanId::new(2)),
            ("finished", PlanId::new(1)),
        ]
    );
}

async fn serve_all(launcher: FakeLauncher, commands: Vec<ApplyCommand>) -> Vec<Report> {
    let (sender, receiver) = unbounded_channel();
    let (reports, queue) = crate::waking::channel(crate::waking::Wake::none());
    for command in commands {
        sender.send(command).unwrap();
    }

    drop(sender);
    tokio::time::timeout(
        Duration::from_secs(20),
        serve(
            receiver,
            Arc::new(launcher),
            Reporter {
                reports,
                gate: OutputGate::default(),
            },
        ),
    )
    .await
    .unwrap();

    queue.try_iter().collect()
}
