use toolbox_plan::protocol::{Mode, TerminalSize};

use super::{FakeLauncher, recording_script, root_plan, run_after};
use crate::apply::ApplyCommand;

#[tokio::test]
async fn a_new_segment_starts_at_the_consoles_size() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = FakeLauncher {
        script: recording_script(&record, 2),
        mode: Mode::Run,
    };

    let size = TerminalSize {
        columns: 120,
        rows: 33,
    };

    run_after(vec![ApplyCommand::Resize(size)], root_plan(), launcher).await;
    let received = std::fs::read_to_string(&record).unwrap();
    let second = received.lines().nth(1).unwrap();

    assert!(
        second.contains("resize") && second.contains("120") && second.contains("33"),
        "{second}"
    );
}
