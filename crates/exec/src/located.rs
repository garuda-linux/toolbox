//! Where a program of a step lies. A release runs the fixed Arch path and
//! nothing else. A development build on NixOS, which has no `/usr/bin`,
//! also looks in the system's profile and its setuid wrappers. Never in
//! the caller's `PATH`, which a root helper must not trust, except for a
//! terminal editor: it only ever runs as the user, in the user's Console,
//! so the user's `PATH` grants it nothing the user lacks.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use toolbox_plan::Editor;
use toolbox_plan::procedure::Program;

const NIXOS: [&str; 2] = ["/run/current-system/sw/bin", "/run/wrappers/bin"];

pub fn program(program: Program) -> PathBuf {
    if Editor::ALL.iter().any(|editor| editor.program() == program)
        && let Some(found) = editor(program)
    {
        return found;
    }

    fixed(program.path())
}

pub fn editor(program: Program) -> Option<PathBuf> {
    let path = std::env::var_os("PATH");
    editor_path(program.path(), path.as_deref(), Path::is_file)
}

pub fn editor_path(
    fixed: &str,
    path: Option<&OsStr>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let fixed = Path::new(fixed);
    if exists(fixed) {
        return Some(fixed.to_path_buf());
    }

    let name = fixed.file_name()?;
    std::env::split_paths(path?)
        .filter(|folder| folder.is_absolute())
        .map(|folder| folder.join(name))
        .find(|candidate| exists(candidate))
}

pub fn fixed(path: &str) -> PathBuf {
    self::path(path, cfg!(debug_assertions), Path::is_file)
}

pub fn path(fixed: &str, development: bool, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let fixed = Path::new(fixed);
    if !development || exists(fixed) {
        return fixed.to_path_buf();
    }

    fixed
        .file_name()
        .and_then(|name| {
            NIXOS
                .iter()
                .map(|folder| Path::new(folder).join(name))
                .find(|candidate| exists(candidate))
        })
        .unwrap_or_else(|| fixed.to_path_buf())
}
