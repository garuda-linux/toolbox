use std::io;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use rustix::fs::Mode;
use toolbox_pty::{Chunk, PtyCommand, Session};

/// Everything the program prints before its terminal closes.
fn printed(command: &PtyCommand) -> io::Result<String> {
    let session = Session::spawn(command)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut out = String::new();

    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        match session.output().recv_timeout(left) {
            Ok(Chunk::Text(text)) => out.push_str(&text),
            Ok(Chunk::Closed) | Err(RecvTimeoutError::Disconnected | RecvTimeoutError::Timeout) => {
                break;
            }
        }
    }

    Ok(out)
}

#[test]
fn a_program_runs_in_the_folder_it_was_given() {
    let folder = tempfile::tempdir().unwrap();
    let canonical = folder.path().canonicalize().unwrap();
    let command = PtyCommand::new("sh")
        .args(["-c", "pwd -P"])
        .in_folder(folder.path());

    assert_eq!(
        printed(&command).unwrap().trim(),
        canonical.display().to_string()
    );
}

#[test]
fn a_program_gets_the_umask_it_was_given() {
    let command = PtyCommand::new("sh")
        .args(["-c", "umask"])
        .umask(Mode::from_raw_mode(0o027));

    assert_eq!(printed(&command).unwrap().trim(), "0027");
}
