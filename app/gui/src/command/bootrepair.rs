//! A click on the Boot Repair page. A repair names the system, its layout
//! and, for a locked one, the passphrase, which goes from here into the
//! plan and nowhere else.

use toolbox_core::features::bootrepair::Target;
use toolbox_plan::{Password, RootLayout, VolumeDevice};

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BootRepairCommand {
    Refresh,
    /// Installs GRUB to `target` again (`Some`), or only writes the boot
    /// menu again (`None`).
    Repair {
        system: VolumeDevice,
        layout: Option<RootLayout>,
        passphrase: Option<Password>,
        target: Option<Target>,
    },
    Forget,
    ShowConsole,
    DismissRefusal,
}

impl From<BootRepairCommand> for Command {
    fn from(command: BootRepairCommand) -> Self {
        Self::App(AppCommand::BootRepair(command))
    }
}
