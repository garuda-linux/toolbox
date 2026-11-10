//! `toolbox-lint`: the comment and shape rules that clippy and rustfmt do
//! not cover, for Rust. Run it from the repository root (default folders
//! `app` and `crates`), or pass files and folders.

mod finding;
mod pattern;
mod rules;
mod source;
#[cfg(test)]
mod tests;
mod words;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use finding::Located;

const DEFAULT_ROOTS: [&str; 2] = ["app", "crates"];
const SKIPPED_DIRS: [&str; 5] = ["target", ".git", ".direnv", "node_modules", "result"];

fn main() -> ExitCode {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let roots = if args.is_empty() {
        DEFAULT_ROOTS.iter().map(PathBuf::from).collect()
    } else {
        args
    };

    match run(&roots) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(error) => {
            // A closed stderr leaves nowhere to report the error.
            writeln!(std::io::stderr(), "toolbox-lint: {error}").ok();
            ExitCode::from(2)
        }
    }
}

fn run(roots: &[PathBuf]) -> Result<usize, String> {
    let mut files = Vec::new();
    for root in roots {
        collect_files(root, &mut files)?;
    }

    files.sort();

    // Missing when the linter runs outside the repository root.
    let domains = fs::read_to_string(rules::domain_registered::REGISTRY)
        .ok()
        .map(|text| rules::domain_registered::registered(&text));

    let mut out = std::io::stdout().lock();
    let mut count = 0;

    for path in &files {
        let text = fs::read_to_string(path)
            .map_err(|err| format!("cannot read {}: {err}", path.display()))?;

        for finding in rules::lint_file_in(path, &text, domains.as_ref()) {
            writeln!(
                out,
                "{}\n",
                Located {
                    path,
                    finding: &finding
                }
            )
            .map_err(|err| format!("cannot write the report: {err}"))?;
            count += 1;
        }
    }

    writeln!(
        std::io::stderr(),
        "toolbox-lint: {count} findings in {} files",
        files.len()
    )
    .ok();
    Ok(count)
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if source::is_linted(path) {
            files.push(path.to_owned());
        }

        return Ok(());
    }

    let entries =
        fs::read_dir(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("cannot read {}: {err}", path.display()))?;
        let child = entry.path();
        let skipped = child
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| SKIPPED_DIRS.contains(&name));

        if !skipped {
            collect_files(&child, files)?;
        }
    }

    Ok(())
}
