use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsoleCommand {
    /// Uploads `0`, the output the user saw in the question and agreed to.
    Share(String),
    HideShare,
    /// Updates the system, then runs the install that met a file its
    /// mirror no longer has again, as one plan.
    UpdateAndRetry,
    /// Empties the terminal once no plan runs.
    ClearOutput,
}

impl From<ConsoleCommand> for Command {
    fn from(command: ConsoleCommand) -> Self {
        Self::App(AppCommand::Console(command))
    }
}
