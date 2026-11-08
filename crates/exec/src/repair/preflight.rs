//! The helper's own check of a boot repair's devices, before any program
//! runs. The value objects only check the shape of a name, so here each
//! must be a block device of the named kind that the running system does
//! not use. A request that fails here is rejected whole.

mod error;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use toolbox_plan::{
    EFI_FIRMWARE, InstalledSystem, PartitionDevice, RepairRoot, RepairWork, RootAction,
};

pub use self::error::PreflightError;
use super::below_repair_root;
use super::disks::{DeviceNumber, Kind, Nodes, disks, kind, mounts, property};

/// The partition type of an EFI system partition on a GPT disk, and its
/// counterpart in an MBR partition table.
const ESP_TYPES: [&str; 2] = ["c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "0xef"];

const ESP_FILE_SYSTEM: &str = "vfat";
const LUKS: &str = "crypto_LUKS";

/// The udev properties of a file system's type and a partition's type.
const FS_TYPE: &str = "ID_FS_TYPE";
const PART_TYPE: &str = "ID_PART_ENTRY_TYPE";

/// The udev property of what a loop device reads from. Its sysfs
/// `backing_file` reads `/sr0` in the live session, not `/dev/sr0`.
const LOOP_BACKING: &str = "ID_LOOP_BACKING_FILENAME";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moment {
    /// Nothing is mounted yet. A mount below the repair's folder is a
    /// leftover of an earlier repair.
    Before,
    /// Right before `grub-install`. The mounts below the repair's folder
    /// are the repair's own.
    During,
}

/// The system the checks look at: `/` and its device nodes in the helper,
/// a temporary folder and a list in tests.
pub struct System<'system> {
    pub root: &'system Path,
    pub nodes: &'system dyn Nodes,
}

/// Checks the devices of `action` when it is a boot repair. Every other
/// action passes.
///
/// # Errors
///
/// The reason, in words for the Console, when a device is no block
/// device, is of another kind than its role needs, or lies on a disk the
/// running system uses.
pub fn check(system: &System<'_>, action: &RootAction) -> Result<(), PreflightError> {
    let RootAction::RepairBoot {
        system: installed,
        work,
        passphrase: _,
    } = action
    else {
        return Ok(());
    };

    devices(system, installed, work, Moment::Before)
}

/// Checks the devices a repair writes to against the running system.
///
/// # Errors
///
/// The reason a device is unusable, in words for the Console.
pub fn devices(
    system: &System<'_>,
    installed: &InstalledSystem,
    work: &RepairWork,
    moment: Moment,
) -> Result<(), PreflightError> {
    let mut targets = vec![root_device(system, installed)?];

    match work {
        RepairWork::RegenerateConfig { esp: None } => {}
        RepairWork::RegenerateConfig { esp: Some(esp) }
        | RepairWork::InstallUefi { esp, id: _ } => {
            targets.push(esp_device(system, esp)?);
        }
        RepairWork::InstallBios { disk } => {
            targets.push(expect(system, disk.as_str(), Kind::Disk)?);
        }
    }

    unused(system, &targets, moment)
}

fn esp_device(system: &System<'_>, esp: &PartitionDevice) -> Result<DeviceNumber, PreflightError> {
    if !system.root.join(EFI_FIRMWARE).is_dir() {
        return Err(PreflightError::NotUefi);
    }

    let device = expect(system, esp.as_str(), Kind::Partition)?;

    if is_esp(system.root, device) {
        Ok(device)
    } else {
        Err(PreflightError::NoEsp(esp.to_string()))
    }
}

fn root_device(
    system: &System<'_>,
    installed: &InstalledSystem,
) -> Result<DeviceNumber, PreflightError> {
    match &installed.root {
        RepairRoot::Open { device } if device.is_mapped() => system
            .nodes
            .block_device(Path::new(device.as_str()), true)
            .ok_or_else(|| PreflightError::NoBlockDevice(device.to_string())),
        RepairRoot::Open { device } => expect(system, device.as_str(), Kind::Partition),
        RepairRoot::Locked { partition } => {
            let device = expect(system, partition.as_str(), Kind::Partition)?;

            if property(system.root, device, FS_TYPE).as_deref() == Some(LUKS) {
                Ok(device)
            } else {
                Err(PreflightError::NoLuks(partition.to_string()))
            }
        }
    }
}

fn expect(system: &System<'_>, path: &str, wanted: Kind) -> Result<DeviceNumber, PreflightError> {
    let device = system
        .nodes
        .block_device(Path::new(path), false)
        .ok_or_else(|| PreflightError::NoBlockDevice(path.to_owned()))?;

    let found = kind(system.root, device).map_err(|error| PreflightError::Lookup {
        path: path.to_owned(),
        error,
    })?;

    if found == wanted {
        Ok(device)
    } else {
        Err(match wanted {
            Kind::Disk => PreflightError::NoWholeDisk(path.to_owned()),
            Kind::Partition | Kind::Stacked => PreflightError::NoPartition(path.to_owned()),
        })
    }
}

fn is_esp(root: &Path, device: DeviceNumber) -> bool {
    let partition_type = property(root, device, PART_TYPE).unwrap_or_default();

    ESP_TYPES.contains(&partition_type.to_lowercase().as_str())
        && property(root, device, FS_TYPE).as_deref() == Some(ESP_FILE_SYSTEM)
}

/// Fails when a disk behind `targets` holds anything the running system
/// has mounted, which is how the live medium and a system that repairs
/// itself show. A mount left below the helper's folder fails too: a
/// second repair on top of it would unmount the wrong one.
fn unused(
    system: &System<'_>,
    targets: &[DeviceNumber],
    moment: Moment,
) -> Result<(), PreflightError> {
    let mut wanted = BTreeSet::new();

    for target in targets {
        wanted.extend(disks(system.root, *target).map_err(PreflightError::TargetDisks)?);
    }

    let table = mounts(system.root).map_err(PreflightError::MountTable)?;

    for mount in table {
        if below_repair_root(&mount.point) {
            if moment == Moment::During {
                continue;
            }

            return Err(PreflightError::Leftover { point: mount.point });
        }

        if !mount.source.starts_with("/dev/") {
            continue;
        }

        // Without its node the mount cannot be placed on a disk, so
        // every disk counts as used.
        let Some(device) = system.nodes.block_device(Path::new(&mount.source), true) else {
            return Err(PreflightError::NoNode {
                source: mount.source,
                point: mount.point,
            });
        };

        // A mounted device the kernel cannot place is treated as in the
        // way: better one refusal too many than a write to a used disk.
        let used = match held(system, device) {
            Ok(used) => used,
            Err(error) => {
                return Err(PreflightError::MountDisks {
                    source: mount.source,
                    error,
                });
            }
        };

        if let Some(disk) = used.intersection(&wanted).next() {
            return Err(PreflightError::InUse {
                disk: disk.clone(),
                source: mount.source,
                point: mount.point,
            });
        }
    }

    Ok(())
}

/// The disks below a mounted device and the media behind the loop
/// devices among them. The live ISO is a loop device on `sr0` or a USB
/// stick, which is never a mount source itself.
fn held(system: &System<'_>, device: DeviceNumber) -> io::Result<BTreeSet<String>> {
    let mut found = disks(system.root, device)?;
    let mut open: Vec<String> = found.iter().cloned().collect();

    while let Some(name) = open.pop() {
        // A backing file instead of a device lies on a file system the
        // mount table names anyway.
        let backing = system
            .nodes
            .block_device(&Path::new("/dev").join(&name), false)
            .and_then(|number| property(system.root, number, LOOP_BACKING))
            .and_then(|file| system.nodes.block_device(Path::new(&file), true));

        let Some(backing) = backing else {
            continue;
        };

        for disk in disks(system.root, backing)? {
            if found.insert(disk.clone()) {
                open.push(disk);
            }
        }
    }

    Ok(found)
}
