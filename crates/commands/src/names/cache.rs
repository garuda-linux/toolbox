//! A completion name list survives for a few seconds on disk: a shell asks
//! again on every Tab, and the system rarely changes between two of them.

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use toolbox_cli::complete::Names;
use toolbox_fs::{Permissions, Replace};

/// The format's version. An older file is ignored, not read.
const VERSION: &str = "1";

/// How long a list stays good, in seconds.
const TTL: u64 = 10;

/// The folder the lists live in, `None` without a home.
pub fn dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;

    Some(base.join("garuda-toolbox").join("completion"))
}

pub fn lookup(names: Names, now: u64) -> Option<Vec<String>> {
    read(&dir()?, names, now)
}

/// Keeps `found` for the next [`TTL`] seconds. A cache that cannot be
/// written changes nothing.
pub fn store(names: Names, now: u64, found: &[String]) {
    if let Some(dir) = dir() {
        write(&dir, names, now, found);
    }
}

/// The seconds since the epoch, or nought when the clock is before it.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn file(names: Names) -> String {
    let stem: String = format!("{names:?}")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    format!("{stem}.names")
}

pub fn read(dir: &Path, names: Names, now: u64) -> Option<Vec<String>> {
    let (written, found) = stored(dir, names)?;
    (now.saturating_sub(written) <= TTL).then_some(found)
}

/// The list in `dir` for `names` when it was written with `stamp`, however
/// long ago. For names that change only when a file they come from does.
pub fn read_stamped(dir: &Path, names: Names, stamp: u64) -> Option<Vec<String>> {
    let (written, found) = stored(dir, names)?;
    (written == stamp).then_some(found)
}

fn stored(dir: &Path, names: Names) -> Option<(u64, Vec<String>)> {
    let text = std::fs::read_to_string(dir.join(file(names))).ok()?;
    let mut lines = text.lines();
    let mut header = lines.next()?.split_whitespace();

    if header.next()? != VERSION {
        return None;
    }

    let written: u64 = header.next()?.parse().ok()?;
    Some((written, lines.map(str::to_owned).collect()))
}

/// Writes the list beside the others, replaced whole so a completion
/// reading meanwhile never sees half of one. A list that cannot be written
/// is read again next time.
pub fn write(dir: &Path, names: Names, now: u64, found: &[String]) {
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }

    let mut text = format!("{VERSION} {now}\n");
    for name in found.iter().filter(|name| !name.contains('\n')) {
        text.push_str(name);
        text.push('\n');
    }

    let path = dir.join(file(names));
    let private = Replace::new(Permissions::Exactly(0o600));

    if let Err(error) = toolbox_fs::replace(&path, text.as_bytes(), &private) {
        tracing::debug!("cannot keep the names in {}: {error}", path.display());
    }
}
