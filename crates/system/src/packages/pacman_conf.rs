//! The few facts of `pacman.conf` the package probes need: where the
//! databases live and which sync databases are configured.
//!
//! Read with an own small parser instead of the `pacman-conf` program (the
//! `pacmanconf` crate runs it): the probes need no servers, mirrors or
//! signature settings, only the repository sections in file order,
//! `RootDir` and `DBPath`. It reads lines the way libalpm's ini parser
//! does: a `#` starts a comment anywhere on a line, and an `Include` is
//! read in place, so a section in an included file counts like one in
//! `pacman.conf` itself.

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ProbeError;
use crate::files;

const DEFAULT_ROOT: &str = "/";
const DEFAULT_DB_PATH: &str = "var/lib/pacman";

/// pacman stops at the same depth, so a file that includes itself ends.
const MAX_INCLUDE_DEPTH: usize = 10;

/// A closure, so `parse` stays free of I/O.
type Includes<'a> = dyn FnMut(&str) -> Vec<String> + 'a;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacmanConf {
    pub root: PathBuf,
    pub db_path: PathBuf,
    /// The repository sections in file order, each once.
    pub repos: Vec<String>,
}

#[derive(Default)]
struct Parser {
    root: Option<PathBuf>,
    db_path: Option<PathBuf>,
    repos: Vec<String>,
    /// The section the next line belongs to. libalpm keeps it across an
    /// include, in both directions.
    section: Option<String>,
}

impl Default for PacmanConf {
    fn default() -> Self {
        parse("")
    }
}

impl Parser {
    fn feed(&mut self, text: &str, depth: usize, includes: &mut Includes<'_>) {
        for raw in text.lines() {
            let line = raw
                .split_once('#')
                .map_or(raw, |(kept, _comment)| kept)
                .trim();

            if line.is_empty() {
                continue;
            }

            if let Some(section) = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
            {
                self.enter(section);
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .map_or((line, ""), |(key, value)| (key.trim(), value.trim()));

            match (key, self.section.as_deref()) {
                ("Include", Some(_)) if !value.is_empty() && depth < MAX_INCLUDE_DEPTH => {
                    for included in includes(value) {
                        self.feed(&included, depth.saturating_add(1), includes);
                    }
                }
                ("RootDir", Some("options")) if !value.is_empty() => {
                    self.root = Some(PathBuf::from(value));
                }
                ("DBPath", Some("options")) if !value.is_empty() => {
                    self.db_path = Some(PathBuf::from(value));
                }
                _ => {}
            }
        }
    }

    fn enter(&mut self, section: &str) {
        if section != "options"
            && is_repo_name(section)
            && !self.repos.iter().any(|repo| repo == section)
        {
            self.repos.push(section.to_owned());
        }

        self.section = Some(section.to_owned());
    }

    fn finish(self) -> PacmanConf {
        let root = self.root.unwrap_or_else(|| PathBuf::from(DEFAULT_ROOT));
        let db_path = self.db_path.unwrap_or_else(|| root.join(DEFAULT_DB_PATH));

        PacmanConf {
            root,
            db_path,
            repos: self.repos,
        }
    }
}

/// The databases the `pacman.conf` at `path` configures, its includes
/// followed. Text is read lossily, one stray byte hides no repository.
///
/// # Errors
///
/// When `path` itself cannot be read. An include that cannot be read is
/// skipped, as pacman only warns about it.
pub fn read(path: &Path) -> Result<PacmanConf, ProbeError> {
    let text = files::read(path)?;
    let mut parser = Parser::default();

    parser.feed(&text, 0, &mut |pattern| {
        expand(pattern)
            .iter()
            .filter_map(|file| match files::read(file) {
                Ok(included) => Some(included),
                Err(error) => {
                    tracing::debug!("pacman.conf include skipped: {error}");
                    None
                }
            })
            .collect()
    });

    Ok(parser.finish())
}

/// Lines it cannot use are skipped, as pacman would refuse them anyway.
pub fn parse(text: &str) -> PacmanConf {
    let mut parser = Parser::default();

    parser.feed(text, 0, &mut |_pattern| Vec::new());
    parser.finish()
}

/// The files an `Include` value names, sorted as `glob` sorts them. A
/// wildcard (`*`, `?`) is understood in the file name, the folder is
/// taken as written. A value without one is the file itself.
fn expand(pattern: &str) -> Vec<PathBuf> {
    let path = Path::new(pattern);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if !name.contains(['*', '?']) {
        return vec![path.to_owned()];
    }

    let folder = path.parent().unwrap_or_else(|| Path::new("/"));
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };

    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|candidate| wildcard_match(name, candidate))
        })
        .map(|entry| entry.path())
        .collect();

    found.sort();
    found
}

/// Whether `name` matches `pattern` with `*` and `?`. Like `glob`, a
/// wildcard never matches a leading dot.
fn wildcard_match(pattern: &str, name: &str) -> bool {
    if name.starts_with('.') && !pattern.starts_with('.') {
        return false;
    }

    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    let (mut at, mut name_at) = (0_usize, 0_usize);
    let mut star: Option<(usize, usize)> = None;

    while name_at < name.len() {
        match pattern.get(at) {
            Some('*') => {
                star = Some((at, name_at));
                at = at.saturating_add(1);
            }
            Some(&wanted) if wanted == '?' || name.get(name_at) == Some(&wanted) => {
                at = at.saturating_add(1);
                name_at = name_at.saturating_add(1);
            }
            _ => match star {
                Some((star_at, star_name)) => {
                    at = star_at.saturating_add(1);
                    name_at = star_name.saturating_add(1);
                    star = Some((star_at, star_name.saturating_add(1)));
                }
                None => return false,
            },
        }
    }

    pattern
        .get(at..)
        .is_some_and(|rest| rest.iter().all(|letter| *letter == '*'))
}

/// libalpm opens `<DBPath>/sync/<name>.db`, so a name must stay one file
/// name. `local` is the local database's own name.
fn is_repo_name(name: &str) -> bool {
    !name.is_empty()
        && name != "local"
        && !name.starts_with('.')
        && name
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '-' | '_' | '.'))
}
