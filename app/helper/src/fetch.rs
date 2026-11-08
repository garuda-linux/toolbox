//! `garuda-toolbox-helper fetch <path>`: the text of one root-owned config
//! file, for the Config Files page. The only command that takes a path,
//! so it trusts nothing about the path: it must be a
//! `ConfigFilePath` (below the folders the toolbox edits, never the
//! password hashes). Its symlinks are resolved, and every step must be
//! such a path again. That real path is then opened without
//! following anything, and only a regular text file of a sane size is
//! handed out. The answer is one JSON line on stdout, with the real path
//! beside the text.
//!
//! A command of its own because polkit tells the helper's commands apart
//! by the first argument: these files can hold what only root may read,
//! so the password-less `read` must never reach them.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;
use std::process::ExitCode;

use toolbox_exec::{ReadError, read_config_file};
use toolbox_plan::protocol::encode;
use toolbox_plan::{ConfigFilePath, FetchAnswer};

use crate::io::{print_answer, refuse};

pub fn run(path: &str) -> ExitCode {
    let Ok(path) = ConfigFilePath::new(path) else {
        return refuse("it does not hand out this file");
    };

    print_answer(
        answer(Path::new("/"), &path),
        &format!("cannot read {path}"),
    )
}

/// # Errors
///
/// When the system refuses the read. A file that is missing, not regular,
/// too large or no text, and a link that leads to no file the toolbox
/// edits, is an answer, not an error.
pub fn answer(root: &Path, path: &ConfigFilePath) -> io::Result<String> {
    let answer = match read_config_file(root, path) {
        Ok(read) => FetchAnswer::Text {
            path: read.path,
            content: read.content,
        },
        Err(ReadError::Refused(reason)) => FetchAnswer::Refused { reason },
        Err(ReadError::Failed(error)) => return Err(error),
    };

    encode(&answer).map_err(io::Error::other)
}
