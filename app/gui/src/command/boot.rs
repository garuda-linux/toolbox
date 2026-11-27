use toolbox_core::features::boot::edit::Edit;
use toolbox_plan::PackageName;
use toolbox_plan::sdboot::LoaderSetting;

use super::{AppCommand, Command};
use crate::boot::FileKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BootCommand {
    Edit(Edit),
    Loader(LoaderSetting),
    Refresh,
    Choose {
        kind: FileKind,
        title: String,
        filter: String,
    },
    /// Opens the page with the kernel offered as the default menu entry.
    Propose(PackageName),
    DropProposal,
}

impl From<BootCommand> for Command {
    fn from(command: BootCommand) -> Self {
        Self::App(AppCommand::Boot(command))
    }
}
