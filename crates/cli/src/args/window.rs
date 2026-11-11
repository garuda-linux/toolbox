//! clap's `args_conflicts_with_subcommands` also refuses the global output
//! flags before a command, so the window flags and the flags of a group's
//! default command are checked here instead.

use std::ffi::OsString;

use clap::error::ErrorKind;
use clap::parser::ValueSource;
use clap::{ArgMatches, FromArgMatches as _};

use super::{Cli, Window};

impl Cli {
    /// The words as the binary parses them, program name first.
    ///
    /// # Errors
    ///
    /// clap's own error for words that are no command line, and a conflict
    /// for a window flag next to a command or a group's flag next to one of
    /// its commands.
    pub fn try_from_words<I, T>(words: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::try_from_matches(&crate::command().try_get_matches_from(words)?)
    }

    /// # Errors
    ///
    /// As [`Cli::try_from_words`].
    pub fn try_from_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        if let Some(conflict) = group_flag_with_a_command(matches) {
            return Err(crate::command().error(ErrorKind::ArgumentConflict, conflict));
        }

        let cli = Self::from_arg_matches(matches)?;

        match (&cli.command, cli.window.flag()) {
            (Some(_), Some(flag)) => Err(crate::command().error(
                ErrorKind::ArgumentConflict,
                format!("{flag} opens the window and goes without a command"),
            )),
            _ => Ok(cli),
        }
    }
}

impl Window {
    const fn flag(&self) -> Option<&'static str> {
        if self.page.is_some() {
            Some("--page")
        } else if self.setup_assistant {
            Some("--setup-assistant")
        } else if self.boot_repair {
            Some("--boot-repair")
        } else {
            None
        }
    }
}

/// `tweaks --json dns`: a flag of the group's default command before
/// another of its commands.
fn group_flag_with_a_command(matches: &ArgMatches) -> Option<String> {
    let (group, inner) = matches.subcommand()?;
    let (named, _) = inner.subcommand()?;

    let mut command = crate::command();
    command.build();
    let group_command = command.find_subcommand(group)?;

    let flag = group_command.get_arguments().find(|arg| {
        !arg.is_global_set()
            && inner.value_source(arg.get_id().as_str()) == Some(ValueSource::CommandLine)
    })?;

    let shown = flag
        .get_long()
        .map_or_else(|| flag.get_id().to_string(), |long| format!("--{long}"));

    Some(format!(
        "{shown} goes with `{group}` alone, not before `{named}`"
    ))
}
