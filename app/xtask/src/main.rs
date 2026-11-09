//! `cargo xtask <task>`: repository tasks for development and CI. Not
//! installed. Paths are relative to the repository root unless given.

mod apps;
mod catalog_ftl;
mod catalogs;
mod cli_docs;
mod icons;
mod translations;

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use regex::Regex;

/// The catalogs besides the toggle catalogs: skel, config files and colour schemes.
const FIXED_CATALOGS: usize = 3;
/// The app's locale folders, relative to the repository root.
const LOCALES_DIR: &str = "app/gui/i18n";
const ENGLISH: &str = "en-US";

/// Name, argument synopsis and function of every task.
const TASKS: [(&str, &str, Task); 7] = [
    ("catalog-ftl", "[--check]", catalog_ftl::catalog_ftl),
    ("catalog-validate", "[dir]", catalog_validate),
    ("cli-docs", "[--check]", cli_docs::cli_docs),
    (
        "i18n-export",
        "[--translations] [--check]",
        translations::i18n_export,
    ),
    (
        "i18n-import",
        "[--check] [--dry-run]",
        translations::i18n_import,
    ),
    ("icons-check", "[theme dir]", icons::icons_check),
    (
        "wiki-sync",
        "[--local <dir>] [--sources <dir>] [--out <dir>] [--include-aur] [--unfiltered] [--write]",
        apps::tasks::wiki_sync,
    ),
];

/// A task: its arguments in, a success message or a [`Failure`] out.
type Task = fn(&[String]) -> Result<String, Failure>;

#[derive(Debug)]
enum Failure {
    /// The arguments do not fit the task.
    Usage,
    /// The task ran and found problems, one line each.
    Problems(Vec<String>),
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args
        .split_first()
        .and_then(|(name, rest)| find(name).map(|task| (task, rest)));

    let result = match task {
        Some((task, rest)) => task(rest),
        None => Err(Failure::Usage),
    };

    // A closed stdout or stderr leaves nowhere to report to.
    match result {
        Ok(message) => {
            writeln!(io::stdout(), "{message}").ok();
            ExitCode::SUCCESS
        }
        Err(Failure::Problems(lines)) => {
            let mut err = io::stderr().lock();

            for line in lines {
                writeln!(err, "{line}").ok();
            }

            ExitCode::FAILURE
        }
        Err(Failure::Usage) => {
            writeln!(io::stderr(), "{}", usage()).ok();
            ExitCode::from(2)
        }
    }
}

fn find(name: &str) -> Option<Task> {
    TASKS
        .iter()
        .find(|(task, _, _)| *task == name)
        .map(|(_, _, task)| *task)
}

fn usage() -> String {
    let tasks: Vec<String> = TASKS
        .iter()
        .map(|(name, synopsis, _)| format!("  cargo xtask {name} {synopsis}"))
        .collect();

    format!("usage:\n{}", tasks.join("\n"))
}

/// Which of `allowed` the arguments name, or `None` for anything else.
fn flags<'a>(args: &[String], allowed: &[&'a str]) -> Option<Vec<&'a str>> {
    args.iter()
        .map(|arg| allowed.iter().find(|flag| **flag == arg).copied())
        .collect()
}

/// The repository root: xtask always runs from the source tree.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A pattern that fails to compile is a mistake in xtask. Skipping it would
/// turn the check that uses it into a silent pass.
#[expect(
    clippy::expect_used,
    reason = "patterns are constants exercised by the tests"
)]
fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid pattern")
}

/// Loads every catalog in `dir` (default `data/catalog`) and reports each
/// broken one.
fn catalog_validate(args: &[String]) -> Result<String, Failure> {
    let (dir, shown) = match args {
        [] => (root().join(catalogs::SOURCE_DIR), catalogs::SOURCE_DIR),
        [dir] => (PathBuf::from(dir), dir.as_str()),
        [_, _, ..] => return Err(Failure::Usage),
    };

    let loaded = catalogs::load(&dir)
        .map_err(|errors| Failure::Problems(errors.iter().map(ToString::to_string).collect()))?;

    let unused = catalogs::unused_writable(&dir, &loaded.config_files);

    if !unused.is_empty() {
        return Err(Failure::Problems(
            unused.iter().map(ToString::to_string).collect(),
        ));
    }

    let files = FIXED_CATALOGS + loaded.toggles.len();

    Ok(format!("{files} catalogs in {shown} are valid"))
}
