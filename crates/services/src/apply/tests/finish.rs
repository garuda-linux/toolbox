use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::unbounded_channel;
use toolbox_plan::protocol::Mode;

use super::{FakeLauncher, recording_script, root_plan};
use crate::apply::{ApplyCommand, Reporter, serve};

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

    let (sender, receiver) = unbounded_channel();
    let (reports, _queue) = crate::waking::channel(crate::waking::Wake::none());
    sender.send(ApplyCommand::Run(root_plan())).unwrap();
    sender.send(ApplyCommand::Finish).unwrap();
    drop(sender);

    tokio::time::timeout(
        Duration::from_secs(20),
        serve(receiver, Arc::new(launcher), Reporter(reports)),
    )
    .await
    .unwrap();

    let sent = std::fs::read_to_string(&record).unwrap();

    assert_eq!(sent.lines().nth(1), Some("{\"type\":\"finish\"}"));
}
