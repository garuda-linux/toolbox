//! Garuda Toolbox's command line, free of egui and of the system.
//!
//! `args` is what the one binary accepts. Without a subcommand it opens
//! the window, with one it answers in the terminal. `exit` names the exit
//! codes scripts can rely on, `output` how answers are printed, and
//! `confirm` the question before a change, behind a prompt port so the
//! tests need no terminal. `complete` answers the shell's Tab with names
//! the app finds, `suggest` the closest name to one the system lacks. The
//! app crate runs the commands, `xtask` makes the completion hooks and
//! the man page.

pub mod args;
pub mod complete;
pub mod confirm;
pub mod exit;
pub mod output;
pub mod suggest;

pub use args::Cli;
use clap::CommandFactory as _;
use clap::builder::StyledStr;
pub use exit::Exit;

/// The command line as the binary parses it.
///
/// clap styles its own headings only, so an "Example:" heading written in
/// an `after_help` text gets the same header style here. The man page uses
/// the plain `Cli::command()`, roff has no use for escape codes.
pub fn command() -> clap::Command {
    styled_examples(Cli::command())
}

fn styled_examples(command: clap::Command) -> clap::Command {
    let palette = output::Paint::help();
    let header = palette.get_header();
    let restyled = command.get_after_help().map(|text| {
        let text = text.to_string();
        let styled = match text.split_once(':') {
            Some((head, rest)) if head.starts_with("Example") => {
                format!("{}{head}:{}{rest}", header.render(), header.render_reset())
            }
            _ => text,
        };

        StyledStr::from(styled)
    });

    let command = match restyled {
        Some(text) => command.after_help(text),
        None => command,
    };

    command.mut_subcommands(styled_examples)
}
