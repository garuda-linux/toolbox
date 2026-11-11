//! The names of all pictures in the icon folders, read once. Most packages
//! have no picture, and `freedesktop-icons` only learns that after trying
//! every theme, size and file ending, about 10 ms a package. A name no
//! folder has misses here at once.

#[cfg(test)]
mod tests;

use std::cell::OnceCell;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

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
pub struct Indexed<L> {
    inner: L,
    names: OnceCell<IconNames>,
    read: Box<dyn Fn() -> IconNames + Send>,
}

impl IconNames {
    /// The folders `freedesktop-icons` searches: `icons` and `pixmaps` of
    /// every XDG data folder, `~/.icons` and `/usr/share/pixmaps`.
    pub fn system() -> Self {
        Self::read(&search_folders())
    }

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
    pub fn new(inner: L, read: impl Fn() -> IconNames + Send + 'static) -> Self {
        Self {
            inner,
            names: OnceCell::new(),
            read: Box::new(read),
        }
    }

    fn names(&self) -> &IconNames {
        self.names.get_or_init(|| {
            let _read = tracing::debug_span!("read", what = "the icon names").entered();
            (self.read)()
        })
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

fn search_folders() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".local/share")));

    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|dirs| !dirs.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_owned());

    let mut folders: Vec<PathBuf> = data_dirs
        .split(':')
        .map(PathBuf::from)
        .chain(data_home)
        .flat_map(|data| [data.join("icons"), data.join("pixmaps")])
        .collect();

    folders.extend(home.map(|home| home.join(".icons")));
    folders.push(PathBuf::from("/usr/share/pixmaps"));
    folders
}
