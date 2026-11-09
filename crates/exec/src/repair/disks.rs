//! What the kernel and udev say about a block device, read from `/dev`,
//! `/sys`, `/run/udev` and `/proc`. A boot repair asks before it touches a
//! disk: the names in a request only say what the app saw, and the helper
//! trusts none of it.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::io;
use std::os::unix::fs::{FileTypeExt as _, MetadataExt as _};
use std::path::{Path, PathBuf};

use toolbox_plan::lines::lossy;

/// How deep device-mapper devices may be stacked before the look gives
/// up: LVM on LUKS on RAID on partitions is four.
const MAX_STACK: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceNumber {
    pub major: u32,
    pub minor: u32,
}

/// What a block device is, by its folder in `/sys`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Disk,
    Partition,
    /// A device-mapper or RAID device that lies on other devices.
    Stacked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mount {
    pub source: String,
    pub point: String,
}

/// Looks at device nodes. Tests answer from a list, since only root can
/// make a block device node.
pub trait Nodes {
    /// The number of the block device at `path`, `None` when there is
    /// none. With `follow` a symlink leads to its target, as the names
    /// below `/dev/mapper` do. Without it only the node itself counts.
    fn block_device(&self, path: &Path, follow: bool) -> Option<DeviceNumber>;
}

pub struct DeviceNodes;

impl Nodes for DeviceNodes {
    fn block_device(&self, path: &Path, follow: bool) -> Option<DeviceNumber> {
        let metadata = if follow {
            std::fs::metadata(path)
        } else {
            std::fs::symlink_metadata(path)
        }
        .ok()?;

        metadata.file_type().is_block_device().then(|| {
            let device = metadata.rdev();

            DeviceNumber {
                major: rustix::fs::major(device),
                minor: rustix::fs::minor(device),
            }
        })
    }
}

fn device_folder(root: &Path, device: DeviceNumber) -> io::Result<PathBuf> {
    root.join(format!("sys/dev/block/{}:{}", device.major, device.minor))
        .canonicalize()
}

fn slaves(folder: &Path) -> io::Result<Vec<PathBuf>> {
    match std::fs::read_dir(folder.join("slaves")) {
        Ok(entries) => entries.map(|entry| entry?.path().canonicalize()).collect(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn kind_of(folder: &Path) -> io::Result<Kind> {
    if folder.join("partition").exists() {
        Ok(Kind::Partition)
    } else if slaves(folder)?.is_empty() {
        Ok(Kind::Disk)
    } else {
        Ok(Kind::Stacked)
    }
}

/// # Errors
///
/// When `/sys` below `root` does not know the device.
pub fn kind(root: &Path, device: DeviceNumber) -> io::Result<Kind> {
    kind_of(&device_folder(root, device)?)
}

/// The kernel names of the whole disks `device` lies on: itself for a
/// disk, the parent of a partition, and those of every device below a
/// stacked one.
///
/// # Errors
///
/// When `/sys` below `root` does not know the device or the stack is
/// deeper than any real one.
pub fn disks(root: &Path, device: DeviceNumber) -> io::Result<BTreeSet<String>> {
    let mut found = BTreeSet::new();

    collect_disks(&device_folder(root, device)?, MAX_STACK, &mut found)?;
    Ok(found)
}

fn collect_disks(folder: &Path, depth: usize, found: &mut BTreeSet<String>) -> io::Result<()> {
    let name_of = |device: &Path| {
        device
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
            .ok_or_else(|| io::Error::other("a device folder without a name"))
    };

    match kind_of(folder)? {
        Kind::Disk => {
            found.insert(name_of(folder)?);
        }
        Kind::Partition => {
            let disk = folder
                .parent()
                .ok_or_else(|| io::Error::other("a partition without a disk"))?;

            found.insert(name_of(disk)?);
        }
        Kind::Stacked => {
            let Some(depth) = depth.checked_sub(1) else {
                return Err(io::Error::other("the devices are stacked too deep"));
            };

            for below in slaves(folder)? {
                collect_disks(&below, depth, found)?;
            }
        }
    }

    Ok(())
}

/// One property udev keeps for `device`, such as `ID_FS_TYPE`. `None`
/// when udev has no record or the record lacks the property.
pub fn property(root: &Path, device: DeviceNumber, key: &str) -> Option<String> {
    let record = lossy(
        std::fs::read(root.join(format!("run/udev/data/b{}:{}", device.major, device.minor)))
            .ok()?,
    );

    record.lines().find_map(|line| {
        let (name, value) = line.strip_prefix("E:")?.split_once('=')?;

        (name == key).then(|| value.to_owned())
    })
}

/// The mount table of the helper's own process.
///
/// # Errors
///
/// When `/proc/self/mountinfo` below `root` cannot be read.
pub fn mounts(root: &Path) -> io::Result<Vec<Mount>> {
    let table = lossy(std::fs::read(root.join("proc/self/mountinfo"))?);

    Ok(table.lines().filter_map(parse_mount).collect())
}

/// `36 35 98:0 / /mnt rw,noatime shared:1 - ext4 /dev/sda2 rw`: the mount
/// point is the fifth field, the source the second one after the `-`.
fn parse_mount(line: &str) -> Option<Mount> {
    let (before, after) = line.split_once(" - ")?;
    let point = before.split(' ').nth(4)?;
    let source = after.split(' ').nth(1)?;

    Some(Mount {
        source: unescape(source),
        point: unescape(point),
    })
}

/// The kernel writes a blank, a tab, a line break and a backslash in a
/// path as `\040`, `\011`, `\012` and `\134`.
fn unescape(field: &str) -> String {
    field
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}
