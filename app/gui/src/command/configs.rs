//! A click on the Config Files page. Rows are named by their key: the
//! catalog entry, or the entry and the file name of a folder's file.

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigsCommand {
    Refresh,
    Open(String),
    Edit(String),
    View(String),
    CloseView,
    Apply,
    EditAgain,
    Reread,
    Discard,
    Dismiss,
    ShowConsole,
    DismissRefusal,
}

impl From<ConfigsCommand> for Command {
    fn from(command: ConfigsCommand) -> Self {
        Self::App(AppCommand::Configs(command))
    }
}
