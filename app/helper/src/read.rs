//! `garuda-toolbox-helper read <kind>` and `logs <kind>`: fixed questions
//! about files and logs only root may read. The caller picks one of a
//! closed list of kinds, never a path or a command, and gets back only
//! what the app shows. The answer is one JSON line on stdout.
//!
//! Two commands, because polkit tells them apart by the first argument:
//! `read` may run without a password in an active session, `logs` (the
//! kernel log and the journal, which the system keeps from ordinary
//! users) never may.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{self, Read as _};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::sync::Arc;

use toolbox_exec::{CaptureError, Captured, Environment, Policy, capture, sdboot_menu};
use toolbox_plan::protocol::{MAX_READ_OUTPUT_BYTES, ProgramAnswer, ReadKind, encode};
use toolbox_plan::{GRUB_CONFIG, GrubMenu, SHADOW, ShadowLocks};

use crate::io::{print_answer, refuse};

/// Far above any real `/etc/shadow` or `grub.cfg`. A larger file is not
/// read at all.
const MAX_FILE_BYTES: u64 = 4 << 20;

pub fn run(command: &str, kind: &str) -> ExitCode {
    let Some(kind) = ReadKind::from_args(command, kind) else {
        return refuse("it does not know this kind of read");
    };

    print_answer(
        answer(Path::new("/"), kind),
        &format!("cannot answer {}", kind.as_str()),
    )
}

/// # Errors
///
/// When the file behind the kind cannot be read, is too large or is no
/// text, or when its program cannot be started. A program that is not
/// installed or exits with an error is an answer, not an error.
pub fn answer(root: &Path, kind: ReadKind) -> io::Result<String> {
    match kind {
        ReadKind::ShadowLocks => read_text(&root.join(SHADOW), |text| {
            encode(&ShadowLocks::from_shadow(text))
        }),
        ReadKind::GrubMenu => read_text(&root.join(GRUB_CONFIG), |text| {
            encode(&GrubMenu::from_config(text))
        }),
        ReadKind::SdBootMenu => encode(&sdboot_menu(root)?).map_err(io::Error::other),
        ReadKind::Dmesg | ReadKind::Journal => {
            encode(&run_program(root, kind)?).map_err(io::Error::other)
        }
    }
}

/// The answer `answer` makes of the text at `path`. Both files may hold
/// password hashes, so their bytes are overwritten once the answer, which
/// holds none, is made.
fn read_text<E>(path: &Path, answer: impl FnOnce(&str) -> Result<String, E>) -> io::Result<String>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let mut content = read_capped(path)?;
    let line = std::str::from_utf8(&content)
        .map_err(|_not_text| io::Error::new(io::ErrorKind::InvalidData, "the file is no text"))
        .and_then(|text| answer(text).map_err(io::Error::other));

    content.fill(0);
    line
}

/// Runs the kind's own program with its own arguments and variables in a
/// clean environment. Nothing of the command comes from the caller.
fn run_program(root: &Path, kind: ReadKind) -> io::Result<ProgramAnswer> {
    let (program, arguments) = kind
        .program()
        .ok_or_else(|| io::Error::other("this kind runs no program"))?;

    let mut command = Command::new(program.below(root));

    command.args(arguments);

    match capture(
        &mut command,
        Environment::Clean,
        kind.environment(),
        &Policy::tail(MAX_READ_OUTPUT_BYTES),
        &Arc::default(),
    ) {
        Ok(Captured {
            output,
            errors,
            code,
            truncated,
        }) => Ok(ProgramAnswer::Ran {
            output,
            errors,
            code,
            truncated,
        }),
        Err(CaptureError::Missing) => Ok(ProgramAnswer::Missing),
        Err(CaptureError::Failed(error)) => Err(error),
        // No cancel and no deadline: this read passes a flag nobody sets
        // and a policy without a time limit.
        Err(CaptureError::Cancelled | CaptureError::TimedOut) => {
            Err(io::Error::other("the read was stopped"))
        }
        // A tail policy keeps the end instead.
        Err(CaptureError::TooLarge) => Err(io::Error::other("the read printed too much")),
    }
}

fn read_capped(path: &Path) -> io::Result<Vec<u8>> {
    let mut content = Vec::new();
    let read = File::open(path)?
        .take(MAX_FILE_BYTES.saturating_add(1))
        .read_to_end(&mut content)?;

    if u64::try_from(read).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        content.fill(0);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the file is too large",
        ));
    }

    Ok(content)
}
