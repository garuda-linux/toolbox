#[cfg(test)]
mod tests;

use std::io::{IsTerminal as _, Write as _};

use toolbox_cli::Exit;
use toolbox_cli::args::{Completions, Help};
use toolbox_cli::output::Style;

use super::out::{say_error, style_on};
use crate::out::Out;

pub fn completions(args: &Completions, out: &mut Out<'_>) -> Exit {
    toolbox_cli::complete::registration(args.shell, out);
    Exit::Success
}

/// The long help of the command the words name, the whole program without
/// words. A word that is no command ends with a usage error.
pub fn help(args: &Help, out: &mut Out<'_>) -> Exit {
    let mut command = toolbox_cli::command();

    for name in &args.command {
        let Some(found) = command.find_subcommand(name).cloned() else {
            say_error(out, &format!("no command `{name}`"));
            return Exit::Usage;
        };

        command = found;
    }

    let style = style_on(out.settings().color, std::io::stdout().is_terminal());
    write!(out, "{}", long_help(&command, style)).ok();
    Exit::Success
}

fn long_help(command: &clap::Command, style: Style) -> String {
    let mut command = command.clone();
    let help = command.render_long_help();

    match style {
        Style::Color => help.ansi().to_string(),
        Style::Plain => help.to_string(),
    }
}
