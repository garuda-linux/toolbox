//! Starts GUI programs as the user, in their own process group, so they
//! outlive a closed app window. A task waits for each one, so finished
//! programs never linger as zombies.

mod stderr;
#[cfg(test)]
mod tests;

use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::{Child, Command};
use tokio::runtime::Handle;
use toolbox_config::Paths;
use toolbox_gui_kit::toasts::ToastKind;

use self::stderr::StderrFile;
use crate::command::{AppCommand, CommandSender};
use crate::spawn::spawn_logged;
use crate::{i18n, log_folder};

/// How many of its last stderr lines a failed program's warning carries.
const TAIL_LINES: usize = 5;

/// Starts programs outside the Console. Dry runs start nothing.
pub trait Launcher: Debug + Send + Sync {
    /// Returns once `program` started.
    ///
    /// # Errors
    ///
    /// When it cannot start, for example because it is not installed.
    fn start(&self, program: &Path, args: &[&str]) -> io::Result<()>;
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
}

/// Logs the start instead, for the dry run and its self-test.
#[derive(Debug)]
pub struct DryRunLauncher;

impl SystemLauncher {
    pub fn new(runtime: Handle, commands: CommandSender, ctx: egui::Context) -> Self {
        let log_dir = log_folder::log_dir(&Paths::from_process());
        Self::logging_to(runtime, commands, ctx, log_dir)
    }

    /// Removes what programs of an earlier start left in `log_dir`.
    pub fn logging_to(
        runtime: Handle,
        commands: CommandSender,
        ctx: egui::Context,
        log_dir: Option<PathBuf>,
    ) -> Self {
        if let Some(dir) = log_dir.clone() {
            drop(runtime.spawn_blocking(move || stderr::sweep(&dir)));
        }

        Self {
            runtime,
            commands,
            ctx,
            log_dir,
        }
    }

    /// The file `short`'s stderr goes to, and its handle for the child.
    fn stderr_for(&self, short: &str) -> (Option<StderrFile>, Stdio) {
        let Some(dir) = &self.log_dir else {
            return (None, Stdio::null());
        };

        match StderrFile::create(dir, short) {
            Ok((file, handle)) => (Some(file), Stdio::from(handle)),
            Err(error) => {
                tracing::warn!("{short} starts without a file for its stderr: {error}");
                (None, Stdio::null())
            }
        }
    }
}

impl Launcher for SystemLauncher {
    fn start(&self, program: &Path, args: &[&str]) -> io::Result<()> {
        let _entered = self.runtime.enter();
        let name = program.display().to_string();
        let short = program
            .file_name()
            .map_or_else(|| name.clone(), |file| file.to_string_lossy().into_owned());

        let (output, stderr) = self.stderr_for(&short);
        let spawned = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
            .process_group(0)
            .spawn();

        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => {
                if let Some(output) = output {
                    drop(self.runtime.spawn_blocking(move || output.remove()));
                }

                return Err(error);
            }
        };

        tracing::info!("started {name}");
        let commands = self.commands.clone();
        let ctx = self.ctx.clone();
        spawn_logged(&self.runtime, "program watch", async move {
            let Some(status) = ended(&mut child, &name).await else {
                return;
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

            let text = i18n::text_with(&ctx, "toast-tool-failed", &[("name", &short)]);
            commands.send(AppCommand::Toast {
                kind: ToastKind::Error,
                text: text.into_owned(),
            });
            ctx.request_repaint();
        });

        Ok(())
    }
}

impl Launcher for DryRunLauncher {
    fn start(&self, program: &Path, _args: &[&str]) -> io::Result<()> {
        tracing::info!("dry run: {} would start now", program.display());
        Ok(())
    }
}

async fn ended(child: &mut Child, name: &str) -> Option<std::process::ExitStatus> {
    child
        .wait()
        .await
        .inspect_err(|error| tracing::warn!("lost track of {name}: {error}"))
        .ok()
}
