//! A started program's stderr goes to a file of its own in the log
//! folder. A pipe would close with the window, and the program's next
//! write would end it by SIGPIPE. A file outlives the window. The
//! program's end reads the last lines and removes the file.

#[cfg(test)]
mod tests;

use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{self, Read as _, Seek as _, SeekFrom};
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The folder below the log folder that holds the files, so the sweep at
/// the start removes nothing else.
const FOLDER: &str = "programs";

/// The end of a file the tail reads. A chatty program may have written
/// far more, its last lines are what tells why it failed.
const TAIL_BYTES: u64 = 16 * 1024;

/// Two programs of the same name started within one second get files of
/// their own.
const ATTEMPTS: u32 = 16;

const EXTENSION: &str = "stderr";

#[derive(Debug)]
pub struct StderrFile {
    path: PathBuf,
}

impl StderrFile {
    /// A new file for `program` below `log_dir`, and the handle the
    /// program writes to. Only the user reads it: a tool may print what
    /// it was given.
    ///
    /// # Errors
    ///
    /// When the folder or the file cannot be made.
    pub fn create(log_dir: &Path, program: &str) -> io::Result<(Self, File)> {
        let dir = log_dir.join(FOLDER);
        DirBuilder::new().recursive(true).mode(0o700).create(&dir)?;

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u64, |since| since.as_secs());

        let mut taken = io::Error::from(io::ErrorKind::AlreadyExists);
        for attempt in 0..ATTEMPTS {
            let name = format!("{}{program}-{stamp}-{attempt}.{EXTENSION}", own_prefix());
            let path = dir.join(name);
            let opened = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path);

            match opened {
                Ok(file) => return Ok((Self { path }, file)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => taken = error,
                Err(error) => return Err(error),
            }
        }

        Err(taken)
    }

    /// The last `count` lines the program wrote, bytes that are no UTF-8
    /// replaced. The file is gone afterwards. Blocks, so it runs on a
    /// blocking thread.
    pub fn finish(self, count: usize) -> Vec<String> {
        let lines = last_lines(&self.path, count).unwrap_or_else(|error| {
            tracing::debug!("cannot read {}: {error}", self.path.display());
            Vec::new()
        });

        self.remove();
        lines
    }

    /// For a program that did not start.
    pub fn remove(self) {
        if let Err(error) = std::fs::remove_file(&self.path) {
            tracing::debug!("cannot remove {}: {error}", self.path.display());
        }
    }
}

/// Removes the files that programs of an earlier start left, as no window
/// waits for them anymore. A program that still runs writes on into its
/// file, which then has no name. The files of this start stay, as the
/// sweep may run after its first program started. Blocks, so it runs on
/// a blocking thread.
pub fn sweep(log_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(log_dir.join(FOLDER)) else {
        return;
    };

    let own = own_prefix();
    for entry in entries.flatten() {
        let path = entry.path();
        let earlier = !entry.file_name().to_string_lossy().starts_with(&own);
        if earlier
            && path
                .extension()
                .is_some_and(|extension| extension == EXTENSION)
            && let Err(error) = std::fs::remove_file(&path)
        {
            tracing::debug!("cannot remove {}: {error}", path.display());
        }
    }
}

fn own_prefix() -> String {
    format!("{}-", std::process::id())
}

fn last_lines(path: &Path, count: usize) -> io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(TAIL_BYTES)))?;

    let mut bytes = Vec::new();
    file.take(TAIL_BYTES).read_to_end(&mut bytes)?;

    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    let skipped = lines.len().saturating_sub(count);
    Ok(lines.into_iter().skip(skipped).map(str::to_owned).collect())
}
