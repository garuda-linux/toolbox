//! Waits for the Console to reach the state a test looks at, never for a
//! fixed time.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tempfile::TempDir;
use toolbox_gui::App;
use toolbox_gui::testing::PATIENCE;

/// Frames for a grid's sizing pass and the last reports after the label.
pub const SETTLE: usize = 8;

/// Holds a script's run until the test opens it, so the run ends after
/// what the test does while it runs, however slow the machine.
pub struct Gate {
    folder: TempDir,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            folder: tempfile::tempdir().unwrap(),
        }
    }

    /// `script` with a wait in front of its output. The wait also ends
    /// when the gate is dropped, so no script outlives its test.
    pub fn hold(&self, script: &str) -> String {
        let folder = self.folder.path().display();
        let open = self.open_file().display().to_string();
        script.replacen(
            "read request;",
            &format!(
                "read request; while [ -d '{folder}' ] && [ ! -e '{open}' ]; do sleep 0.01; done;"
            ),
            1,
        )
    }

    pub fn open(&self) {
        std::fs::write(self.open_file(), "").unwrap();
    }

    fn open_file(&self) -> PathBuf {
        self.folder.path().join("open")
    }
}

/// Steps frames until `label` shows, then lets the layout settle. The run
/// happens on the runtime, so a frame only sees it once its reports
/// arrived. Single steps, because the service's repaints may land in the
/// middle of `Harness::run` and push it past its step limit under load.
pub fn wait_for(harness: &mut Harness<'static, App>, label: &str) {
    let start = Instant::now();
    while harness.query_by_label(label).is_none() {
        assert!(start.elapsed() < PATIENCE, "“{label}” never showed");
        std::thread::sleep(Duration::from_millis(20));
        harness.step();
    }

    harness.run_steps(SETTLE);
}

/// Takes the keyboard focus off the Console, for a picture of what it
/// shows. Which header control the opening frame focuses depends on
/// whether the run's first steps had arrived by then, so its ring would
/// come and go between runs.
pub fn unfocus(harness: &mut Harness<'static, App>) {
    harness.ctx.memory_mut(|memory| {
        if let Some(focused) = memory.focused() {
            memory.surrender_focus(focused);
        }
    });
    harness.run_steps(SETTLE);
}
