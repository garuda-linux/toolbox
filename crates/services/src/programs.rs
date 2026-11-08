//! The programs the live ISO's rows on Home and the Maintenance page need.
//! Checked on a blocking thread at start and after every plan, which may
//! have installed one.

#[cfg(test)]
mod tests;

use std::path::Path;

use toolbox_core::features::live::INSTALLER;
use toolbox_core::features::maintenance::page::{
    GARUDA_UPDATE_PROGRAM, RepairRow, ToolRow, repairs, tools,
};
use toolbox_plan::procedure::Program;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Programs {
    pub installer: bool,
    /// garuda-chroot, for a shell in the installed system.
    pub chroot: bool,
    /// garuda-update, which the Rescue repairs and the upgrade prefer.
    pub garuda_update: bool,
    /// The tools Maintenance opens, each with whether it is installed.
    pub tools: Vec<ToolRow>,
    /// The repairs of Maintenance's Rescue, each with whether it can run.
    pub repairs: Vec<RepairRow>,
}

impl Programs {
    pub fn check() -> Self {
        Self::found(Path::exists)
    }

    pub fn found(exists: impl Fn(&Path) -> bool) -> Self {
        Self {
            installer: exists(Path::new(INSTALLER)),
            chroot: exists(Path::new(Program::GarudaChroot.path())),
            garuda_update: exists(Path::new(GARUDA_UPDATE_PROGRAM)),
            tools: tools(&exists),
            repairs: repairs(&exists),
        }
    }
}
