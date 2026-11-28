//! What the user may read without root, asked from the files themselves,
//! and the end of pacman's log.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{self, Read as _};
use std::path::Path;

use toolbox_state::diagnostics::{Access, Attempt, Failure, Ran, since_last_pacman_run};

use crate::status::pacman_log::{PACMAN_LOG, whole_lines};

const DMESG_RESTRICT: &str = "proc/sys/kernel/dmesg_restrict";
const MACHINE_ID: &str = "etc/machine-id";
/// Persistent first, then the journal of systems that keep it in memory.
const JOURNAL_FOLDERS: [&str; 2] = ["var/log/journal", "run/log/journal"];

/// What the user may read below `root` without the helper.
///
/// The kernel log is open when `kernel.dmesg_restrict` is 0 (Arch sets 1).
/// The journal is open when the system journal's file can be opened, which
/// covers the groups `systemd-journal`, `adm` and `wheel` and any ACL
/// without knowing them. Where nothing says otherwise the user's own
/// rights are tried, so root is never asked on a guess.
pub fn access(root: &Path) -> Access {
    let kernel_log = std::fs::read_to_string(root.join(DMESG_RESTRICT))
        .map_or(true, |value| value.trim() == "0");

    Access {
        kernel_log,
        journal: journal_is_open(root),
    }
}

/// The part of pacman's log since its last run, at most the last `limit`
/// bytes of the file. Every user may read the log on Arch (mode 0644), so
/// no password is asked.
pub fn pacman_log(root: &Path, limit: usize) -> Attempt {
    let path = root.join(PACMAN_LOG);

    match read_tail(&path, limit) {
        Ok((text, cut)) => {
            let (part, found) = since_last_pacman_run(&text);

            Attempt::Ran(Ran {
                output: part.to_owned(),
                errors: String::new(),
                code: Some(0),
                // A run found inside the kept part makes the cut irrelevant.
                truncated: cut && !found,
            })
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Attempt::Missing,
        Err(error) => Attempt::Failed(Failure::Read {
            path: path.display().to_string(),
            error: error.to_string(),
        }),
    }
}

fn journal_is_open(root: &Path) -> bool {
    let Ok(machine) = std::fs::read_to_string(root.join(MACHINE_ID)) else {
        return true;
    };

    JOURNAL_FOLDERS
        .iter()
        .map(|folder| {
            root.join(folder)
                .join(machine.trim())
                .join("system.journal")
        })
        .find_map(|file| match File::open(&file) {
            Ok(_) => Some(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(_) => Some(false),
        })
        .unwrap_or(true)
}

/// The last `limit` bytes of a file as text, and whether its start was
/// left out. A cut text starts at its first whole line.
fn read_tail(path: &Path, limit: usize) -> io::Result<(String, bool)> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    let limit = u64::try_from(limit).unwrap_or(u64::MAX);
    let cut = length > limit;
    let mut bytes = Vec::new();

    whole_lines(&mut file, length, length.saturating_sub(limit))?.read_to_end(&mut bytes)?;

    Ok((String::from_utf8_lossy(&bytes).into_owned(), cut))
}
