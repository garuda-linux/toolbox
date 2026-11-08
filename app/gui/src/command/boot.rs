use toolbox_core::features::boot::edit::Edit;
use toolbox_plan::PackageName;
use toolbox_plan::sdboot::LoaderSetting;

use super::{AppCommand, Command};
use crate::boot::FileKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BootCommand {
    Edit(Edit),
    Loader(LoaderSetting),
    DismissRefusal,
    Refresh,
    Choose {
        kind: FileKind,
        title: String,
        filter: String,
    },
    /// Opens the page with the menu entry of `0` offered as the default.
    Propose(PackageName),
    DropProposal,
}

impl From<BootCommand> for Command {
    fn from(command: BootCommand) -> Self {
        Self::App(AppCommand::Boot(command))
    }
}
