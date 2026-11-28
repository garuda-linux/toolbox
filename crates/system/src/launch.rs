//! Starting a program as the user, in its own process group, and waiting for
//! its end. The caller decides where its stderr goes, a program's stdin and
//! stdout are closed.

#[cfg(test)]
mod tests;

use std::fmt::Debug;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::{ExitStatus, Stdio};

use futures_lite::future::Boxed;
use tokio::process::Command;

/// Why a program could not be run to its end.
#[derive(Debug)]
pub enum RunError {
    /// The program did not start, as its file is missing or not executable.
    NotStarted(io::Error),
    /// It started, but its end could not be waited for.
    Lost(io::Error),
}

/// Runs programs. Tests answer from a fake.
pub trait ProgramRunner: Debug + Send + Sync {
    /// Starts `program` with `args` and returns its exit status once it ends.
    fn run(
        &self,
        program: &Path,
        args: &[String],
        stderr: Option<File>,
    ) -> Boxed<Result<ExitStatus, RunError>>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemRunner;

impl ProgramRunner for SystemRunner {
    fn run(
        &self,
        program: &Path,
        args: &[String],
        stderr: Option<File>,
    ) -> Boxed<Result<ExitStatus, RunError>> {
        let program = program.to_path_buf();
        let args = args.to_vec();
        Box::pin(async move { run(&program, &args, stderr).await })
    }
}

async fn run(
    program: &Path,
    args: &[String],
    stderr: Option<File>,
) -> Result<ExitStatus, RunError> {
    let name = program.display().to_string();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr.map_or_else(Stdio::null, Stdio::from))
        .process_group(0)
        .spawn()
        .map_err(RunError::NotStarted)?;

    tracing::info!("started {name}");
    child.wait().await.map_err(RunError::Lost)
}
