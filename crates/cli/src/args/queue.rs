//! A queued change is typed as its own command after `queue add`, so every change command, its checks and its help serve the queue too.

use clap::{Args, CommandFactory as _, Subcommand};

use super::{Cli, Command, Confirm, Read};

/// Longer selectors name no queued change.
const SELECTOR_MAX: usize = 512;

#[derive(Debug, Subcommand)]
pub enum QueueCommand {
    /// Show the queued changes, numbered, and the plan that carries them
    /// out.
    #[command(after_help = "Example:\n  garuda-toolbox queue show --json")]
    Show(Read),

    /// Queue a change instead of carrying it out now. Type the change as
    /// its own command.
    #[command(
        after_help = "Examples:\n  garuda-toolbox queue add toggle on pipewire-support\n  \
                      garuda-toolbox queue add services enable sshd.service\n  \
                      garuda-toolbox queue add users lock guest"
    )]
    Add(QueueAdd),

    /// Take back one queued change, by its number in `queue show` or by
    /// its name.
    #[command(after_help = "Examples:\n  garuda-toolbox queue remove 2\n  \
                      garuda-toolbox queue remove firefox")]
    Remove(QueueRemove),

    /// Take back every queued change.
    #[command(after_help = "Example:\n  garuda-toolbox queue clear --yes")]
    Clear(QueueClear),

    /// Carry out the whole queue as one plan, with one password.
    #[command(after_help = "Examples:\n  garuda-toolbox queue apply --dry-run\n  \
                      garuda-toolbox queue apply --yes")]
    Apply(Confirm),
}

#[derive(Debug, Args)]
pub struct QueueAdd {
    /// The change as its own command takes it, for example `toggle on
    /// pipewire-support`. Only changes the window queues fit: switches,
    /// apps, kernels, services at boot and masks, locales, accounts,
    /// groups, boot settings, drivers, DKMS builds and maintenance tasks.
    #[arg(
        required = true,
        num_args = 1..,
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "CHANGE"
    )]
    pub change: Vec<String>,
}

#[derive(Debug, Args)]
pub struct QueueRemove {
    /// The change's number in `queue show`, or its name there, such as a
    /// package or a unit.
    #[arg(value_parser = selector)]
    pub change: String,
}

#[derive(Clone, Copy, Debug, Args)]
pub struct QueueClear {
    /// Do not ask. Needed without a terminal.
    #[arg(long, short = 'y')]
    pub yes: bool,
}

impl QueueAdd {
    /// The words after `queue add` as the command they type.
    ///
    /// # Errors
    ///
    /// clap's own error for words that are no command.
    pub fn command(&self) -> Result<Command, clap::Error> {
        let words = std::iter::once("garuda-toolbox").chain(self.change.iter().map(String::as_str));
        let cli = Cli::try_from_words(words)?;

        cli.command.ok_or_else(|| {
            Cli::command().error(
                clap::error::ErrorKind::MissingSubcommand,
                "`queue add` wants a change command, such as `toggle on pipewire-support`",
            )
        })
    }
}

fn selector(text: &str) -> Result<String, String> {
    let valid = (1..=SELECTOR_MAX).contains(&text.len()) && !text.chars().any(char::is_control);

    if valid {
        Ok(text.to_owned())
    } else {
        Err(format!(
            "`{text}` names no queued change: a number or a name, at most {SELECTOR_MAX} bytes"
        ))
    }
}
