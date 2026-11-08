//! Where a config file really is. Distributions put symlinks into `/etc`,
//! and an edit of such a link edits the file it leads to. The links are
//! followed here, once and by hand. Every link on the way and the result
//! must be a `ConfigFilePath`. Reading and writing take that real path and
//! follow nothing, so a link swapped in after this look fails the request.

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::{fs, io};

use toolbox_plan::{ConfigFilePath, FetchRefusal};

/// As many links as the kernel follows before it answers `ELOOP`.
const MAX_LINKS: usize = 40;

/// The page shows a refused target in one line.
const MAX_SHOWN_CHARACTERS: usize = 1024;

/// A path without a symlink on its way, at the time it was looked at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealPath {
    pub path: ConfigFilePath,
    /// The path that was asked for led here through a symlink.
    pub linked: bool,
}

#[derive(Debug)]
pub enum Unresolved {
    Missing,
    /// A symlink that leads nowhere the toolbox edits. `target` is where
    /// it leads as far as it could be followed, for showing only.
    Refused {
        reason: FetchRefusal,
        target: String,
    },
    /// The system refused the look, for the app usually because a folder
    /// on the way is root's alone.
    Failed(io::Error),
}

struct Walk {
    /// Below the root, none of them a symlink.
    done: Vec<String>,
    pending: VecDeque<String>,
    links: usize,
    /// The path that was asked for is a symlink itself, so whatever is
    /// missing from here on is its target and not the file.
    dangles: bool,
}

impl Walk {
    fn step(&mut self, root: &Path, part: String) -> Result<(), Unresolved> {
        match part.as_str() {
            "" | "." => return Ok(()),
            ".." => {
                self.done.pop();
                return Ok(());
            }
            _ => {}
        }

        let place = self.below(root, &part);
        let metadata = match fs::symlink_metadata(&place) {
            Ok(metadata) => metadata,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                return Err(self.nowhere(&part));
            }
            Err(error) => return Err(Unresolved::Failed(error)),
        };

        if !metadata.file_type().is_symlink() {
            self.done.push(part);
            return Ok(());
        }

        // A link is only followed where the toolbox edits, which root
        // owns. One in a folder another user can write would let that
        // user pick the target of an administrator's edit.
        if ConfigFilePath::new(self.place(&part)).is_err() {
            return Err(refused(FetchRefusal::Outside, &self.shown(&part)));
        }

        self.dangles |= self.pending.is_empty();
        self.links += 1;

        if self.links > MAX_LINKS {
            return Err(refused(FetchRefusal::Dangling, &self.shown(&part)));
        }

        let target = fs::read_link(&place).map_err(Unresolved::Failed)?;
        let Some(target) = target.to_str() else {
            return Err(refused(FetchRefusal::Outside, &target.to_string_lossy()));
        };

        if target.starts_with('/') {
            self.done.clear();
        }

        for piece in target.rsplit('/') {
            self.pending.push_front(piece.to_owned());
        }

        Ok(())
    }

    fn below(&self, root: &Path, part: &str) -> PathBuf {
        let mut place = root.to_path_buf();

        place.extend(&self.done);
        place.push(part);
        place
    }

    fn place(&self, part: &str) -> String {
        let folder: String = self.done.iter().flat_map(|piece| ["/", piece]).collect();

        format!("{folder}/{part}")
    }

    fn shown(&self, part: &str) -> String {
        let parts: Vec<&str> = self
            .done
            .iter()
            .map(String::as_str)
            .chain([part])
            .chain(self.pending.iter().map(String::as_str))
            .filter(|piece| !piece.is_empty())
            .collect();

        format!("/{}", parts.join("/"))
    }

    fn nowhere(&self, part: &str) -> Unresolved {
        if self.dangles {
            refused(FetchRefusal::Dangling, &self.shown(part))
        } else {
            Unresolved::Missing
        }
    }
}

/// Follows the symlinks of `path` below `root`, like the kernel would in a
/// system whose `/` is `root`: an absolute target starts at `root` again
/// and `..` never climbs above it.
///
/// # Errors
///
/// [`Unresolved::Missing`] when the path does not exist,
/// [`Unresolved::Refused`] for a link that dangles, runs in a circle or
/// leads to what is no [`ConfigFilePath`], [`Unresolved::Failed`] when the
/// system refuses the look.
pub fn resolve(root: &Path, path: &ConfigFilePath) -> Result<RealPath, Unresolved> {
    let mut walk = Walk {
        done: Vec::new(),
        pending: path.relative().split('/').map(str::to_owned).collect(),
        links: 0,
        dangles: false,
    };

    while let Some(part) = walk.pending.pop_front() {
        walk.step(root, part)?;
    }

    let real = format!("/{}", walk.done.join("/"));

    match ConfigFilePath::new(real.as_str()) {
        Ok(real_path) => Ok(RealPath {
            path: real_path,
            linked: walk.links > 0,
        }),
        Err(_outside) => Err(refused(FetchRefusal::Outside, &real)),
    }
}

/// The target is text of the file system: it is shown, so it loses what
/// could break a line or act as a code.
fn refused(reason: FetchRefusal, target: &str) -> Unresolved {
    Unresolved::Refused {
        reason,
        target: target
            .chars()
            .take(MAX_SHOWN_CHARACTERS)
            .map(|character| {
                if character.is_control() {
                    char::REPLACEMENT_CHARACTER
                } else {
                    character
                }
            })
            .collect(),
    }
}
