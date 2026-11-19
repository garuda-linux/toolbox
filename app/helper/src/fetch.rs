//! `garuda-toolbox-helper fetch <path>`: the text of one root-owned file,
//! for the Config Files page and the merges of pacman's copies. The only
//! command that takes a path, so it trusts nothing about the path: it must
//! be a `ConfigFilePath` (below the folders the toolbox edits, never the
//! password hashes) that the catalog edits or shows, or a copy pacman left
//! beside a backup file and that file. A catalog path's symlinks are
//! resolved, and every step must be such a path again, the real path a
//! listed one too. That real path is then opened without following
//! anything, and only a regular text file of a sane size is handed out.
//! The answer is one JSON line on stdout, with the real path beside the
//! text.
//!
//! A command of its own because polkit tells the helper's commands apart
//! by the first argument: these files can hold what only root may read,
//! so the password-less `read` must never reach them.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;
use std::process::ExitCode;

use toolbox_exec::{ConfigRead, ReadError, is_fetchable, read_config_file, read_real};
use toolbox_plan::protocol::encode;
use toolbox_plan::{ConfigFilePath, FetchAnswer, FetchRefusal};

use crate::io::{print_answer, refuse};

pub fn run(path: &str) -> ExitCode {
    let root = Path::new("/");
    let Ok(path) = ConfigFilePath::new(path) else {
        return refuse("it does not hand out this file");
    };

    let answered = if path.is_readable() {
        answer(root, &path)
    } else if is_fetchable(root, &path) {
        merge_answer(root, &path)
    } else {
        return refuse("it does not hand out this file");
    };

    print_answer(answered, &format!("cannot read {path}"))
}

/// # Errors
///
/// When the system refuses the read. A file that is missing, not regular,
/// too large or no text, and a link that leads to no file the toolbox
/// edits or shows, is an answer, not an error.
pub fn answer(root: &Path, path: &ConfigFilePath) -> io::Result<String> {
    encoded(read_config_file(root, path).and_then(|read| {
        // `run` checked the asked path only, and a link of a listed folder
        // may lead to a secret no deny list names.
        if read.path.is_readable() {
            Ok(read)
        } else {
            Err(ReadError::Refused(FetchRefusal::Outside))
        }
    }))
}

/// A copy pacman left, or the backup file beside it. No catalog entry
/// names them, so no symlink is followed at all.
///
/// # Errors
///
/// Like [`answer`].
pub fn merge_answer(root: &Path, path: &ConfigFilePath) -> io::Result<String> {
    encoded(read_real(root, path).map(|content| ConfigRead {
        path: path.clone(),
        content,
    }))
}

fn encoded(read: Result<ConfigRead, ReadError>) -> io::Result<String> {
    let answer = match read {
        Ok(read) => FetchAnswer::Text {
            path: read.path,
            content: read.content,
        },
        Err(ReadError::Refused(reason)) => FetchAnswer::Refused { reason },
        Err(ReadError::Failed(error)) => return Err(error),
    };

    encode(&answer).map_err(io::Error::other)
}
