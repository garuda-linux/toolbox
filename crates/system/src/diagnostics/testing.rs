use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use tempfile::TempDir;
use toolbox_plan::protocol::ReadKind;
use toolbox_state::diagnostics::{Collector, Outcome};

use super::{DiagnosticSource as _, HelperReads, SystemDiagnostics};
use crate::test_support::{runnable_script, write};

pub struct FakeHelper {
    script: PathBuf,
    asked: Mutex<Vec<ReadKind>>,
}

impl HelperReads for FakeHelper {
    fn read_command(&self, kind: ReadKind) -> Command {
        self.asked.lock().push(kind);

        let mut command = Command::new(&self.script);

        command.args([kind.command(), kind.as_str()]);
        command
    }
}

pub struct Machine {
    dir: TempDir,
    helper: Arc<FakeHelper>,
}

impl Machine {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();

        fs::create_dir_all(dir.path().join("bin")).unwrap();

        let helper = Arc::new(FakeHelper {
            script: dir.path().join("helper"),
            asked: Mutex::new(Vec::new()),
        });

        Self { dir, helper }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn file(&self, path: &str, content: &str) {
        write(self.root(), path, content);
    }

    pub fn program(&self, name: &str, script: &str) {
        runnable_script(&self.root().join("bin").join(name), script);
    }

    pub fn helper(&self, script: &str) {
        runnable_script(&self.helper.script, script);
    }

    pub fn collect(&self, collector: Collector) -> Outcome {
        self.source().collect(collector, &Arc::default()).unwrap()
    }

    pub fn source(&self) -> SystemDiagnostics {
        let helper: Arc<dyn HelperReads> = Arc::<FakeHelper>::clone(&self.helper);

        SystemDiagnostics::new(self.root(), &self.root().join("bin"), helper)
    }

    pub fn asked(&self) -> Vec<ReadKind> {
        self.helper.asked.lock().clone()
    }
}

pub fn report(text: &str) -> Outcome {
    Outcome::Report(text.to_owned())
}

pub fn cancel_after(delay: Duration) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel);

    std::thread::Builder::new()
        .name("cancel".to_owned())
        .spawn(move || {
            std::thread::sleep(delay);
            flag.store(true, Ordering::Relaxed);
        })
        .unwrap();

    cancel
}
