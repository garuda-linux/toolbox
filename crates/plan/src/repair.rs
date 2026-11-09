//! What a boot repair names: the installed system it works on, where the
//! boot loader goes, and the places the helper mounts that system at. The
//! mount points and the name of the unlocked device are constants here, so
//! no request can choose them.

mod esp;
#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

pub use self::esp::{EspPlace, InstalledLoader};
use crate::{BootloaderId, DiskDevice, PartitionDevice, VolumeDevice};

/// Where the helper mounts the installed system, relative to `/`. It lies
/// in root's own `/run`, which is gone after a restart.
pub const REPAIR_ROOT: &str = "run/garuda-toolbox/boot-repair/root";

/// The device-mapper name an unlocked partition gets for one repair.
pub const REPAIR_MAPPER: &str = "garuda-toolbox-repair";

/// The device behind [`REPAIR_MAPPER`].
pub const REPAIR_VOLUME: &str = "/dev/mapper/garuda-toolbox-repair";

/// The subvolume Garuda and most btrfs installs keep the system in.
pub const ROOT_SUBVOLUME: &str = "@";

/// Where an installed system mounts its EFI system partition, relative to
/// its root. Garuda's installer uses this place.
pub const ESP_FOLDER: &str = "boot/efi";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RepairRoot {
    /// A partition or mapped device with the file system on it.
    Open { device: VolumeDevice },
    /// A LUKS partition. The helper unlocks it for the repair and closes
    /// it again in the same step.
    Locked { partition: PartitionDevice },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RootLayout {
    /// The system lies at the top of the file system.
    Plain,
    /// The system lies in the btrfs subvolume [`ROOT_SUBVOLUME`].
    Subvolume,
}

impl RootLayout {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Subvolume => "subvolume",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledSystem {
    pub root: RepairRoot,
    pub layout: RootLayout,
}

impl InstalledSystem {
    /// Whether the helper has to unlock the root first, which takes a
    /// passphrase.
    pub const fn is_locked(&self) -> bool {
        matches!(self.root, RepairRoot::Locked { partition: _ })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RepairWork {
    /// Only the boot menu is generated again. systemd-boot keeps its images
    /// on `esp`, which a UEFI machine names. GRUB does not mount it.
    RegenerateConfig { esp: Option<PartitionDevice> },
    /// GRUB goes to the EFI system partition and into the firmware's boot
    /// entries, then the menu is generated.
    InstallUefi {
        esp: PartitionDevice,
        id: BootloaderId,
    },
    /// GRUB goes to the first sectors of a whole disk, then the menu is
    /// generated.
    InstallBios { disk: DiskDevice },
}

/// What the helper looks for in the mounted system before it changes
/// anything there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairNeed {
    Menu,
    /// systemd-boot builds its images onto the EFI system partition.
    MenuWithEsp,
    Installer,
    /// The installer and the folder for the EFI system partition.
    InstallerWithEsp,
}

impl RepairWork {
    pub const fn need(&self) -> RepairNeed {
        match self {
            Self::RegenerateConfig { esp: None } => RepairNeed::Menu,
            Self::RegenerateConfig { esp: Some(_) } => RepairNeed::MenuWithEsp,
            Self::InstallUefi { esp: _, id: _ } => RepairNeed::InstallerWithEsp,
            Self::InstallBios { disk: _ } => RepairNeed::Installer,
        }
    }
}

/// The places a repair mounts something at. A closed list of paths below
/// [`REPAIR_ROOT`]: no request names a mount point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairMount {
    /// The root of the installed system.
    Root,
    Dev,
    Proc,
    Sys,
    Run,
    EfiVariables,
    Esp(EspPlace),
}

impl RepairMount {
    pub const ALL: [Self; 9] = [
        Self::Root,
        Self::Dev,
        Self::Proc,
        Self::Sys,
        Self::Run,
        Self::EfiVariables,
        Self::Esp(EspPlace::Efi),
        Self::Esp(EspPlace::Boot),
        Self::Esp(EspPlace::BootEfi),
    ];

    /// Where the kernel's views of the live system are bound, in mount
    /// order.
    pub const BINDS: [Self; 4] = [Self::Dev, Self::Proc, Self::Sys, Self::Run];

    /// The place inside the installed system, empty for its root.
    pub const fn inside(self) -> &'static str {
        match self {
            Self::Root => "",
            Self::Dev => "dev",
            Self::Proc => "proc",
            Self::Sys => "sys",
            Self::Run => "run",
            Self::EfiVariables => "sys/firmware/efi/efivars",
            Self::Esp(place) => place.inside(),
        }
    }
}

impl fmt::Display for RepairMount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{REPAIR_ROOT}")?;

        match self.inside() {
            "" => Ok(()),
            inside => write!(f, "/{inside}"),
        }
    }
}
