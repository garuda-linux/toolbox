//! Starting the helper process and reading what it wrote to stderr.

use std::os::fd::OwnedFd;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::net::UnixStream;
use tokio::process::Child;
use tokio::task::JoinHandle;

use super::super::Launcher;

/// What the classifier of pkexec's refusals needs is in the first lines.
const KEPT_STDERR_BYTES: usize = 4096;

/// The program that would not start, so a missing pkexec says which one.
#[derive(Debug, thiserror::Error)]
#[error("{program}: {source}")]
pub(super) struct NotStarted {
    program: String,
    source: std::io::Error,
}

/// A socket, not a pipe: the helper reads the caller's identity from it.
pub(super) fn spawn(launcher: &dyn Launcher) -> Result<(Child, UnixStream), NotStarted> {
    let mut command = launcher.command();
    let program = command
        .as_std()
        .get_program()
        .to_string_lossy()
        .into_owned();

    let failed = |source| NotStarted {
        program: program.clone(),
        source,
    };

    let (ours, theirs) = std::os::unix::net::UnixStream::pair().map_err(failed)?;
    ours.set_nonblocking(true).map_err(failed)?;
    let ours = UnixStream::from_std(ours).map_err(failed)?;

    let child = command
        .stdin(Stdio::from(OwnedFd::from(theirs)))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Reaches pkexec while it still asks. The root helper cannot be
        // signalled by the user: it ends because its stdin closes.
        .kill_on_drop(true)
        .spawn()
        .map_err(failed)?;

    Ok((child, ours))
}

/// Logs the helper's stderr and keeps its first bytes for [`refusal`].
pub(super) fn forward_stderr(child: &mut Child) -> Option<JoinHandle<String>> {
    let stderr = child.stderr.take()?;

    Some(tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut kept = String::new();

        while let Ok(Some(line)) = lines.next_line().await {
            tracing::warn!("helper: {line}");

            if kept.len().saturating_add(line.len()) < KEPT_STDERR_BYTES {
                kept.push_str(&line);
                kept.push('\n');
            }
        }

        kept
    }))
}
