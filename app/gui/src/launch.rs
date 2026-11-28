//! Starts GUI programs as the user, in their own process group, so they
//! outlive a closed app window.
//!
//! The start and the wait run on the runtime, so the frame never waits for
//! a file or a process. A failed start is a toast, and so is a program that
//! ends with an error.

mod stderr;
#[cfg(test)]
mod tests;

use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_config::Paths;
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_system::{ProgramRunner, RunError, SystemRunner};

use self::stderr::StderrFile;
use crate::command::{AppCommand, CommandSender};
use crate::i18n::Arg;
use crate::spawn::spawn_logged;
use crate::{i18n, log_folder};

/// How many of its last stderr lines a failed program's warning carries.
const TAIL_LINES: usize = 5;

/// Starts programs outside the Console. Dry runs start nothing.
pub trait Launcher: Debug + Send + Sync {
    /// Starts `program` in the background and returns at once. A start
    /// that fails says so in a toast.
    fn start(&self, program: &Path, args: &[&str]);
}

/// A program that ends with an error says so in the log, with its last
/// stderr lines, and in a toast with its name.
#[derive(Debug)]
pub struct SystemLauncher {
    runtime: Handle,
    commands: CommandSender,
    ctx: egui::Context,
    /// Where each program's stderr goes. `None` without a log folder, the
    /// programs' stderr is dropped then.
    log_dir: Option<PathBuf>,
    runner: Arc<dyn ProgramRunner>,
}

/// Logs the start instead, for the dry run and its self-test.
#[derive(Debug)]
pub struct DryRunLauncher;

impl SystemLauncher {
    pub fn new(runtime: Handle, commands: CommandSender, ctx: egui::Context) -> Self {
        let log_dir = log_folder::log_dir(&Paths::from_process());
        Self::logging_to(runtime, commands, ctx, log_dir)
    }

    pub fn logging_to(
        runtime: Handle,
        commands: CommandSender,
        ctx: egui::Context,
        log_dir: Option<PathBuf>,
    ) -> Self {
        Self::with_runner(runtime, commands, ctx, log_dir, Arc::new(SystemRunner))
    }

    /// Starts the programs through `runner`, which the self-test and the
    /// tests replace. Removes what programs of an earlier start left in
    /// `log_dir`.
    pub fn with_runner(
        runtime: Handle,
        commands: CommandSender,
        ctx: egui::Context,
        log_dir: Option<PathBuf>,
        runner: Arc<dyn ProgramRunner>,
    ) -> Self {
        if let Some(dir) = log_dir.clone() {
            drop(runtime.spawn_blocking(move || stderr::sweep(&dir)));
        }

        Self {
            runtime,
            commands,
            ctx,
            log_dir,
            runner,
        }
    }
}

/// One start, with what it needs moved onto the runtime.
struct Start {
    program: PathBuf,
    args: Vec<String>,
    log_dir: Option<PathBuf>,
    commands: CommandSender,
    ctx: egui::Context,
    runner: Arc<dyn ProgramRunner>,
}

impl Launcher for SystemLauncher {
    fn start(&self, program: &Path, args: &[&str]) {
        let start = Start {
            program: program.to_path_buf(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            log_dir: self.log_dir.clone(),
            commands: self.commands.clone(),
            ctx: self.ctx.clone(),
            runner: Arc::clone(&self.runner),
        };

        spawn_logged(&self.runtime, "program start", start.run());
    }
}

impl Start {
    async fn run(self) {
        let name = self.program.display().to_string();
        let short = self
            .program
            .file_name()
            .map_or_else(|| name.clone(), |file| file.to_string_lossy().into_owned());

        let (output, stderr) = self.stderr_file(&short).await;
        let status = match self.runner.run(&self.program, &self.args, stderr).await {
            Ok(status) => status,
            Err(RunError::NotStarted(error)) => {
                tracing::error!("cannot start {name}: {error}");
                if let Some(output) = output {
                    drop(tokio::task::spawn_blocking(move || output.remove()));
                }

                let reason = error.to_string();
                self.toast(
                    "toast-tool-not-started",
                    &[("name", Arg::from(&short)), ("reason", Arg::from(&reason))],
                );
                return;
            }
            Err(RunError::Lost(error)) => {
                tracing::warn!("lost track of {name}: {error}");
                return;
            }
        };

        let last = match output {
            Some(output) => tokio::task::spawn_blocking(move || output.finish(TAIL_LINES))
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        };

        if status.success() {
            tracing::debug!("{name} ended with {status}");
            return;
        }

        tracing::warn!(
            "{name} ended with {status}, its last lines: {}",
            last.join("\n")
        );
        self.toast("toast-tool-failed", &[("name", Arg::from(&short))]);
    }

    /// The file `short`'s stderr goes to, and its handle for the program. The
    /// file is made on a blocking thread.
    async fn stderr_file(&self, short: &str) -> (Option<StderrFile>, Option<File>) {
        let Some(dir) = self.log_dir.clone() else {
            return (None, None);
        };

        let name = short.to_owned();
        let created = tokio::task::spawn_blocking(move || StderrFile::create(&dir, &name)).await;
        match created {
            Ok(Ok((file, handle))) => (Some(file), Some(handle)),
            Ok(Err(error)) => {
                tracing::warn!("{short} starts without a file for its stderr: {error}");
                (None, None)
            }
            Err(error) => {
                tracing::warn!("{short} starts without a file for its stderr: {error}");
                (None, None)
            }
        }
    }

    fn toast(&self, id: &str, args: &[(&'static str, Arg<'_>)]) {
        let text = i18n::text_with(&self.ctx, id, args);
        self.commands.send(AppCommand::Toast {
            kind: ToastKind::Error,
            text: text.into_owned(),
        });
        self.ctx.request_repaint();
    }
}

impl Launcher for DryRunLauncher {
    fn start(&self, program: &Path, _args: &[&str]) {
        tracing::info!("dry run: {} would start now", program.display());
    }
}
