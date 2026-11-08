//! The Boot repair page's look at the disks: what they hold and how the
//! machine started.

pub mod lsblk;
#[cfg(any(test, feature = "selftest"))]
pub mod sample;

pub use lsblk::{InvalidListing, LSBLK_ARGUMENTS, parse_lsblk};
use toolbox_plan::{DiskDevice, PartitionDevice, RepairRoot, RootLayout};

const BTRFS: &str = "btrfs";

/// How the running system was started, which is how GRUB must be
/// installed to be found again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Firmware {
    Uefi,
    Bios,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Lsblk,
    GrubInstall,
    Cryptsetup,
}

impl Tool {
    pub const ALL: [Self; 3] = [Self::Lsblk, Self::GrubInstall, Self::Cryptsetup];

    pub const fn path(self) -> &'static str {
        match self {
            Self::Lsblk => "/usr/bin/lsblk",
            Self::GrubInstall => "/usr/bin/grub-install",
            Self::Cryptsetup => "/usr/bin/cryptsetup",
        }
    }

    pub fn name(self) -> &'static str {
        self.path().rsplit('/').next().unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encryption {
    None,
    Locked,
    Unlocked,
}

impl Encryption {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Locked => "locked",
            Self::Unlocked => "unlocked",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct System {
    pub root: RepairRoot,
    pub label: String,
    /// In bytes, 0 when lsblk did not say.
    pub size: u64,
    /// Empty for a locked root: what is inside shows only once it is open.
    pub file_system: String,
    pub encryption: Encryption,
    /// The whole disks it lies on, more than one for a volume group.
    pub disks: Vec<DiskDevice>,
    /// The running system has something of its disk mounted.
    pub in_use: bool,
}

impl System {
    /// The device path, which is also how the page names the system.
    pub fn device(&self) -> &str {
        match &self.root {
            RepairRoot::Open { device } => device.as_str(),
            RepairRoot::Locked { partition } => partition.as_str(),
        }
    }

    /// Whether the page asks where the system lies in the file system:
    /// on btrfs it may be in the `@` subvolume or at the top, and a
    /// locked root does not show its file system at all.
    pub fn asks_layout(&self) -> bool {
        self.encryption == Encryption::Locked || self.file_system == BTRFS
    }

    /// The layout a repair uses when the page asked for none: Garuda's
    /// `@` subvolume wherever one can be.
    pub fn usual_layout(&self) -> RootLayout {
        if self.asks_layout() {
            RootLayout::Subvolume
        } else {
            RootLayout::Plain
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Esp {
    pub device: PartitionDevice,
    pub label: String,
    /// In bytes, 0 when lsblk did not say.
    pub size: u64,
    pub disk: DiskDevice,
    pub in_use: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disk {
    pub device: DiskDevice,
    pub model: String,
    /// In bytes, 0 when lsblk did not say.
    pub size: u64,
    pub in_use: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    pub systems: Vec<System>,
    pub esps: Vec<Esp>,
    pub disks: Vec<Disk>,
}

impl Inventory {
    pub fn system(&self, device: &str) -> Option<&System> {
        self.systems.iter().find(|system| system.device() == device)
    }

    /// The EFI system partition a reinstall for `system` would take
    /// first: one on a disk of the system, else the first free one.
    pub fn usual_esp(&self, system: &System) -> Option<&Esp> {
        let free = || self.esps.iter().filter(|esp| !esp.in_use);

        free()
            .find(|esp| system.disks.contains(&esp.disk))
            .or_else(|| free().next())
    }

    /// The disk a BIOS reinstall for `system` would take first.
    pub fn usual_disk(&self, system: &System) -> Option<&Disk> {
        let free = || self.disks.iter().filter(|disk| !disk.in_use);

        free()
            .find(|disk| system.disks.contains(&disk.device))
            .or_else(|| free().next())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairFacts {
    pub firmware: Firmware,
    pub missing: Vec<Tool>,
    pub inventory: Inventory,
}
