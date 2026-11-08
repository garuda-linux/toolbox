//! The Boot repair page: what the disks of the machine hold, and the one
//! root action a repair is. The page is for the live ISO, where the
//! installed system lies on another disk than the running one.
//!
//! Nothing is queued: a repair runs at once, after the page's confirm
//! dialog named the partition and the disk that are written.

pub mod page;
#[cfg(test)]
mod tests;

use std::sync::LazyLock;

use toolbox_plan::{
    BootloaderId, DiskDevice, InstalledSystem, PartitionDevice, Password, Plan, RepairWork,
    RootAction, RootLayout, VolumeDevice,
};
pub use toolbox_state::bootrepair::{
    Disk, Encryption, Esp, Firmware, InvalidListing, Inventory, LSBLK_ARGUMENTS, RepairFacts,
    System, Tool, parse_lsblk,
};

/// The boot entry name of Garuda's installer, so a reinstall replaces its
/// entry instead of adding one.
#[expect(clippy::expect_used, reason = "a constant name, parsed by a test")]
pub static BOOTLOADER_ID: LazyLock<BootloaderId> =
    LazyLock::new(|| BootloaderId::new("Garuda").expect("`Garuda` is a boot entry name"));

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    NotOffered,
    /// The disks were not read, or a needed program is missing.
    Unavailable,
    UnknownSystem,
    /// The system or the target lies on a disk the running system uses.
    InUse,
    UnknownTarget,
    UnknownLayout,
    /// A locked system came without a passphrase. The code of
    /// `InvalidPassword::Empty`, so the page shows one text for both.
    #[strum(serialize = "empty")]
    NoPassphrase,
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// What the page asks a repair for: the system by its device, the layout
/// (`None` for the usual one) and the passphrase, which is only read for
/// a locked system.
pub struct Wish {
    pub system: VolumeDevice,
    pub layout: Option<RootLayout>,
    pub passphrase: Option<Password>,
}

/// Where a reinstall puts the boot loader: an EFI system partition when
/// the machine started through UEFI, a whole disk otherwise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Esp(PartitionDevice),
    Disk(DiskDevice),
}

impl Target {
    pub fn device(&self) -> &str {
        match self {
            Self::Esp(device) => device.as_str(),
            Self::Disk(device) => device.as_str(),
        }
    }
}

/// # Errors
///
/// [`Refusal`] when the system is not one of `facts` or cannot be used.
pub fn plan_regenerate(facts: &RepairFacts, wish: Wish) -> Result<Plan, Refusal> {
    let (system, passphrase) = chosen(facts, wish)?;

    Ok(plan(
        system,
        RepairWork::RegenerateConfig { esp: None },
        passphrase,
    ))
}

/// # Errors
///
/// [`Refusal`] when the system or the target is not one of `facts` or
/// cannot be used, or the target does not fit how the machine started.
pub fn plan_reinstall(facts: &RepairFacts, wish: Wish, target: &Target) -> Result<Plan, Refusal> {
    let work = match (facts.firmware, target) {
        (Firmware::Uefi, Target::Esp(device)) => {
            let esp = facts
                .inventory
                .esps
                .iter()
                .find(|esp| esp.device == *device)
                .ok_or(Refusal::UnknownTarget)?;

            if esp.in_use {
                return Err(Refusal::InUse);
            }

            RepairWork::InstallUefi {
                esp: esp.device.clone(),
                id: BOOTLOADER_ID.clone(),
            }
        }
        (Firmware::Bios, Target::Disk(device)) => {
            let disk = facts
                .inventory
                .disks
                .iter()
                .find(|disk| disk.device == *device)
                .ok_or(Refusal::UnknownTarget)?;

            if disk.in_use {
                return Err(Refusal::InUse);
            }

            RepairWork::InstallBios {
                disk: disk.device.clone(),
            }
        }
        (Firmware::Uefi, Target::Disk(_)) | (Firmware::Bios, Target::Esp(_)) => {
            return Err(Refusal::UnknownTarget);
        }
    };

    let (system, passphrase) = chosen(facts, wish)?;

    Ok(plan(system, work, passphrase))
}

/// The system the wish names, with its layout and, for a locked one, its
/// passphrase. The passphrase of an open system is dropped unread, and so
/// is a layout of a system that has only one.
fn chosen(facts: &RepairFacts, wish: Wish) -> Result<(InstalledSystem, Option<Password>), Refusal> {
    if !facts.missing.is_empty() {
        return Err(Refusal::Unavailable);
    }

    let system = facts
        .inventory
        .system(wish.system.as_str())
        .ok_or(Refusal::UnknownSystem)?;

    if system.in_use {
        return Err(Refusal::InUse);
    }

    let layout = wish
        .layout
        .filter(|_| system.asks_layout())
        .unwrap_or_else(|| system.usual_layout());

    let passphrase = match system.encryption {
        Encryption::Locked => Some(wish.passphrase.ok_or(Refusal::NoPassphrase)?),
        Encryption::None | Encryption::Unlocked => None,
    };

    Ok((
        InstalledSystem {
            root: system.root.clone(),
            layout,
        },
        passphrase,
    ))
}

fn plan(system: InstalledSystem, work: RepairWork, passphrase: Option<Password>) -> Plan {
    Plan::root(RootAction::RepairBoot {
        system,
        work,
        passphrase,
    })
}
