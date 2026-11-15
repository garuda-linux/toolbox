//! The names of all pictures in the icon folders, read once and again
//! after a plan. Most packages have no picture, and `freedesktop-icons`
//! only learns that after trying every theme, size and file ending, about
//! 10 ms a package. A name no folder has misses here at once.

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;

use super::IconLookup;

/// The endings `freedesktop-icons` tries.
const ENDINGS: [&str; 3] = ["png", "svg", "xpm"];

/// Themes nest `size/context/file`, or `context/size/file`, a few more
/// for some. Deeper is no theme.
const MAX_DEPTH: usize = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IconNames(HashSet<String>);

/// `inner`, asked only for names some folder has. The names are read at
/// the first lookup, on the thread that looks up.
///
/// Shared by the threads that look up app pictures, so the folders are
/// walked once for all.
pub struct Indexed<L> {
    inner: L,
    names: Mutex<Option<Arc<IconNames>>>,
    read: Box<dyn Fn() -> IconNames + Send + Sync>,
}

impl IconNames {
    /// Every picture's stem below `folders`. A linked folder is followed
    /// once, so a link back up ends.
    pub fn read(folders: &[PathBuf]) -> Self {
        let mut names = HashSet::new();
        let mut seen = HashSet::new();
        for folder in folders {
            collect(folder, 0, &mut seen, &mut names);
        }

        Self(names)
    }

    pub fn has(&self, name: &str) -> bool {
        self.0.contains(name)
    }
}

impl<L> Indexed<L> {
    pub fn new(inner: L, read: impl Fn() -> IconNames + Send + Sync + 'static) -> Self {
        Self {
            inner,
            names: Mutex::new(None),
            read: Box::new(read),
        }
    }

    fn names(&self) -> Arc<IconNames> {
        let mut names = self.names.lock();
        let read = Arc::clone(names.get_or_insert_with(|| {
            let _read = tracing::debug_span!("read", what = "the icon names").entered();
            Arc::new((self.read)())
        }));

        drop(names);
        read
    }
}

impl<L: IconLookup> IconLookup for Indexed<L> {
    fn find(&self, theme: &str, name: &str) -> Option<PathBuf> {
        // A slash makes it a file's path to `freedesktop-icons`.
        if name.contains('/') || self.names().has(name) {
            return self.inner.find(theme, name);
        }

        None
    }

    fn forget(&self) {
        *self.names.lock() = None;
        self.inner.forget();
    }
}

fn collect(folder: &Path, depth: usize, seen: &mut HashSet<PathBuf>, names: &mut HashSet<String>) {
    let Ok(real) = fs::canonicalize(folder) else {
        return;
    };

    if depth > MAX_DEPTH || !seen.insert(real) {
        return;
    }

    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, depth + 1, seen, names);
            continue;
        }

        if let Some(stem) = picture_stem(&path) {
            names.insert(stem.to_owned());
        }
    }
}

fn picture_stem(path: &Path) -> Option<&str> {
    let ending = path.extension().and_then(OsStr::to_str)?;
    if !ENDINGS.contains(&ending) {
        return None;
    }

    path.file_stem().and_then(OsStr::to_str)
}
