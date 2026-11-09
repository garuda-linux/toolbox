//! Starts GUI programs as the user, in their own process group, so they
//! outlive a closed app window. A task waits for each one, so finished
//! programs never linger as zombies.

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fmt::Debug;
use std::io;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::process::{Child, ChildStderr, Command};
use tokio::runtime::Handle;
use toolbox_gui_kit::toasts::ToastKind;

use crate::command::{AppCommand, CommandSender};
use crate::i18n;
use crate::spawn::spawn_logged;

/// How many of its last stderr lines a failed program's warning carries.
const TAIL_LINES: usize = 5;

/// How long the stderr reader may still take once the program ended. A
/// child the program left behind can hold stderr open for good.
const TAIL_GRACE: Duration = Duration::from_millis(200);

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
}

/// Logs the start instead, for the dry run and its self-test.
#[derive(Debug)]
pub struct DryRunLauncher;

#[derive(Debug, Default)]
struct Tail(VecDeque<String>);

impl SystemLauncher {
    pub const fn new(runtime: Handle, commands: CommandSender, ctx: egui::Context) -> Self {
        Self {
            runtime,
            commands,
            ctx,
        }
    }
}

impl Launcher for SystemLauncher {
    fn start(&self, program: &Path, args: &[&str]) -> io::Result<()> {
        let _entered = self.runtime.enter();
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()?;

        let name = program.display().to_string();
        let short = program
            .file_name()
            .map_or_else(|| name.clone(), |file| file.to_string_lossy().into_owned());

        tracing::info!("started {name}");
        let tail = child
            .stderr
            .take()
            .map(|stderr| self.runtime.spawn(tail(stderr)));

        let commands = self.commands.clone();
        let ctx = self.ctx.clone();
        spawn_logged(&self.runtime, "program watch", async move {
            let Some(status) = ended(&mut child, &name).await else {
                return;
            };

            if status.success() {
                tracing::debug!("{name} ended with {status}");
                return;
            }

            let last = match tail {
                Some(reader) => tokio::time::timeout(TAIL_GRACE, reader)
                    .await
                    .ok()
                    .and_then(Result::ok)
                    .unwrap_or_default(),
                None => Tail::default(),
            };

            tracing::warn!(
                "{name} ended with {status}, its last lines: {}",
                last.text()
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

impl Tail {
    fn push(&mut self, line: String) {
        if self.0.len() == TAIL_LINES {
            drop(self.0.pop_front());
        }

        self.0.push_back(line);
    }

    fn text(&self) -> String {
        Vec::from_iter(self.0.iter().map(String::as_str)).join("\n")
    }
}

async fn ended(child: &mut Child, name: &str) -> Option<std::process::ExitStatus> {
    child
        .wait()
        .await
        .inspect_err(|error| tracing::warn!("lost track of {name}: {error}"))
        .ok()
}

/// Reads stderr to its end, so a chatty program never blocks on a full
/// pipe, and keeps the last lines.
async fn tail(stderr: ChildStderr) -> Tail {
    let mut lines = BufReader::new(stderr).lines();
    let mut tail = Tail::default();
    while let Ok(Some(line)) = lines.next_line().await {
        tail.push(line);
    }

    tail
}
