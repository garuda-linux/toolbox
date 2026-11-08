//! `icons-check`: every icon name the catalogs give the theme exists in
//! Breeze. A missing name draws a fallback or a look-alike from another
//! set, which reads as a broken icon.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use crate::{Failure, catalogs, regex, root};

/// What a theme name looks like. Files, addresses and sentences do not.
static THEME_NAME: LazyLock<Regex> = LazyLock::new(|| regex("^[a-z][a-z0-9-]*[a-z0-9]$"));

static CATALOG_ICON: LazyLock<Regex> = LazyLock::new(|| regex(r#"^\s*icon\s*=\s*"([^"]*)""#));

/// The theme names of a catalog file: an `icon` with a file ending names a
/// file of the app instead.
pub fn catalog_names(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| CATALOG_ICON.captures(line)?.get(1))
        .map(|name| name.as_str())
        .filter(|name| THEME_NAME.is_match(name))
        .map(ToOwned::to_owned)
        .collect()
}

fn files_below(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut folders = vec![dir.to_path_buf()];

    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let path = entry.path();

            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                folders.push(path);
            } else {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

pub fn theme_names(dir: &Path) -> BTreeSet<String> {
    files_below(dir)
        .iter()
        .filter_map(|path| icon_stem(path))
        .collect()
}

fn icon_stem(path: &Path) -> Option<String> {
    let ending = path.extension()?.to_str()?;

    if !matches!(ending, "svg" | "svgz" | "png") {
        return None;
    }

    path.file_stem()?.to_str().map(ToOwned::to_owned)
}

pub fn check(used: &[(String, String)], theme: &BTreeSet<String>) -> Vec<String> {
    used.iter()
        .filter(|(_, name)| !theme.contains(name))
        .map(|(place, name)| format!("{place}: `{name}` is not in the Breeze icon theme"))
        .collect()
}

/// The Breeze folder: the argument, or the first `icons/breeze` of
/// `XDG_DATA_DIRS` (the dev shell has breeze-icons there).
fn theme_dir(args: &[String]) -> Result<PathBuf, Failure> {
    match args {
        [dir] => Ok(PathBuf::from(dir)),
        [] => std::env::var("XDG_DATA_DIRS")
            .unwrap_or_default()
            .split(':')
            .map(|dir| Path::new(dir).join("icons/breeze"))
            .find(|dir| dir.join("index.theme").is_file())
            .ok_or_else(|| {
                Failure::Problems(vec![
                    "no Breeze icon theme in XDG_DATA_DIRS: run inside `nix develop` or pass its folder"
                        .to_owned(),
                ])
            }),
        [_, _, ..] => Err(Failure::Usage),
    }
}

/// `icons-check [theme dir]`: every icon name in the catalogs is in the
/// Breeze theme.
pub fn icons_check(args: &[String]) -> Result<String, Failure> {
    let theme = theme_names(&theme_dir(args)?);
    let catalog = root().join(catalogs::SOURCE_DIR);
    let mut used = Vec::new();

    for file in files_below(&catalog) {
        if file.extension().is_none_or(|ending| ending != "toml") {
            continue;
        }

        let shown = file
            .strip_prefix(&catalog)
            .unwrap_or(&file)
            .display()
            .to_string();

        let text = fs::read_to_string(&file)
            .map_err(|error| Failure::Problems(vec![format!("{shown}: {error}")]))?;

        used.extend(
            catalog_names(&text)
                .into_iter()
                .map(|name| (shown.clone(), name)),
        );
    }

    let missing = check(&used, &theme);

    if missing.is_empty() {
        Ok(format!(
            "{} icon names are in the Breeze icon theme",
            used.len()
        ))
    } else {
        Err(Failure::Problems(missing))
    }
}
