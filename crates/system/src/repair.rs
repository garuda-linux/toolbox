//! What the Boot repair page reads: the disks from `lsblk`, how the
//! machine was started and which of the needed programs are installed.
//! All of it as the user, and none of it writes.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::PathBuf;

use toolbox_plan::EFI_FIRMWARE;
use toolbox_state::bootrepair::{
    Firmware, Inventory, LSBLK_ARGUMENTS, RepairFacts, Tool, parse_lsblk,
};

use crate::command;
use crate::error::ProbeError;

pub trait RepairSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Failed`] when `lsblk` fails or prints something
    /// else than a device list.
    fn facts(&self) -> Result<RepairFacts, ProbeError>;
}

/// A bare-metal laptop as the live ISO sees it, for the dry run of a self-test
/// build, whose page must neither depend on the disks of the machine it
/// runs on nor show them.
#[cfg(any(test, feature = "selftest"))]
pub struct DryRunRepair;

pub struct SystemRepair {
    root: PathBuf,
    lsblk: OsString,
}

impl SystemRepair {
    #[must_use = "the source does nothing until it is asked"]
    pub fn system() -> Self {
        Self {
            root: PathBuf::from("/"),
            lsblk: Tool::Lsblk.path().into(),
        }
    }

    fn missing(&self) -> Vec<Tool> {
        Tool::ALL
            .into_iter()
            .filter(|tool| {
                !self
                    .root
                    .join(tool.path().trim_start_matches('/'))
                    .is_file()
            })
            .collect()
    }
}

impl RepairSource for SystemRepair {
    fn facts(&self) -> Result<RepairFacts, ProbeError> {
        let missing = self.missing();
        let firmware = if self.root.join(EFI_FIRMWARE).is_dir() {
            Firmware::Uefi
        } else {
            Firmware::Bios
        };

        // Without every program the page offers nothing, so the disks are
        // not worth a read.
        let inventory = if missing.is_empty() {
            let listing = command::output(&self.lsblk, &LSBLK_ARGUMENTS)?;

            parsed(&listing)?
        } else {
            Inventory::default()
        };

        Ok(RepairFacts {
            firmware,
            missing,
            inventory,
        })
    }
}

#[cfg(any(test, feature = "selftest"))]
impl RepairSource for DryRunRepair {
    fn facts(&self) -> Result<RepairFacts, ProbeError> {
        Ok(RepairFacts {
            firmware: Firmware::Uefi,
            missing: Vec::new(),
            inventory: parsed(&toolbox_state::bootrepair::sample::laptop_from_the_live_iso())?,
        })
    }
}

fn parsed(listing: &str) -> Result<Inventory, ProbeError> {
    parse_lsblk(listing).map_err(|error| ProbeError::Failed(error.to_string()))
}
