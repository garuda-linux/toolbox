//! `cli-docs [--check]`: the shell completion hooks and the man pages of
//! the one binary, the pages made from its arguments in `toolbox-cli`.
//! They are checked in under `data/cli/`, so packaging needs no build of
//! the app, and `--check` keeps them in step. A hook asks the installed
//! binary for its completions, which knows the names the system has now.
//! `garuda-toolbox completions <shell>` prints the same hook.

mod man;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;

use clap::CommandFactory as _;
use clap_complete::Shell;
use toolbox_cli::Cli;

use crate::{Failure, flags, root};

const DIR: &str = "data/cli";
const BIN: &str = "garuda-toolbox";

pub fn cli_docs(args: &[String]) -> Result<String, Failure> {
    let check = !flags(args, &["--check"]).ok_or(Failure::Usage)?.is_empty();

    write_or_check(&root().join(DIR), check)
}

/// Writes the files below `dir`, or with `check` only compares them.
fn write_or_check(dir: &Path, check: bool) -> Result<String, Failure> {
    let made = files();

    if check {
        let stale: Vec<String> = made
            .iter()
            .filter(|(name, text)| {
                fs::read(dir.join(name)).ok().as_deref() != Some(text.as_slice())
            })
            .map(|(name, _)| {
                format!(
                    "{}/{name} is out of date, run `cargo xtask cli-docs`",
                    dir.display()
                )
            })
            .collect();

        return if stale.is_empty() {
            Ok(format!("{} command line files are up to date", made.len()))
        } else {
            Err(Failure::Problems(stale))
        };
    }

    fs::create_dir_all(dir).map_err(|error| Failure::Problems(vec![error.to_string()]))?;
    for (name, text) in &made {
        fs::write(dir.join(name), text).map_err(|error| {
            Failure::Problems(vec![format!("{}/{name}: {error}", dir.display())])
        })?;
    }

    Ok(format!(
        "wrote {} command line files to {}",
        made.len(),
        dir.display()
    ))
}

/// Every file with its name: the completion hooks of bash, fish and zsh,
/// the man page, and the page of every command below it.
fn files() -> Vec<(String, Vec<u8>)> {
    let shells = [
        (Shell::Bash, format!("{BIN}.bash")),
        (Shell::Fish, format!("{BIN}.fish")),
        (Shell::Zsh, format!("_{BIN}")),
    ];

    let mut made: Vec<(String, Vec<u8>)> = shells
        .into_iter()
        .map(|(shell, name)| {
            let mut text = Vec::new();
            toolbox_cli::complete::registration(shell, &mut text);
            (name, text)
        })
        .collect();

    // The exit codes belong in the man page, the help leaves them out.
    let top = Cli::command().after_long_help(format!(
        "{}\n\n{}",
        toolbox_cli::args::EXAMPLES,
        toolbox_cli::Exit::table()
    ));

    made.extend(man::pages(top));
    made
}
