//! A click on the Diagnostics page.

use toolbox_core::features::diagnostics::Selection;

use super::{AppCommand, Command};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticsCommand {
    /// Starts a collection of the selected parts.
    Collect(Selection),
    Cancel,
    /// Uploads the shown report, once the user agreed.
    Upload,
    /// Hides the last upload's link or failure.
    HideShare,
}

impl From<DiagnosticsCommand> for Command {
    fn from(command: DiagnosticsCommand) -> Self {
        Self::App(AppCommand::Diagnostics(command))
    }
}
