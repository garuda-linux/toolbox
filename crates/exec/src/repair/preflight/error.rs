//! Why the helper's look at the devices refuses a boot repair. The text is
//! what the Console and the reject event show.

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum PreflightError {
    /// A UEFI install without `/sys/firmware/efi`.
    #[error(
        "the live system was not started through UEFI, so GRUB cannot be registered with the firmware"
    )]
    NotUefi,
    #[error("{0} is no block device")]
    NoBlockDevice(String),
    #[error("{0} is no whole disk")]
    NoWholeDisk(String),
    #[error("{0} is no partition")]
    NoPartition(String),
    #[error("{0} is no EFI system partition")]
    NoEsp(String),
    #[error("{0} is no LUKS partition")]
    NoLuks(String),
    /// The kernel's record of the device at `path` cannot be read.
    #[error("cannot look up {path} in /sys: {error}")]
    Lookup {
        path: String,
        #[source]
        error: io::Error,
    },
    /// The whole disks below a target cannot be found.
    #[error("cannot find the disk of a device in /sys: {0}")]
    TargetDisks(#[source] io::Error),
    #[error("cannot read the mount table: {0}")]
    MountTable(#[source] io::Error),
    /// A mount at or below the repair's folder, left by an earlier run.
    #[error(
        "{point} is still mounted from an earlier repair, unmount it or restart the live system"
    )]
    Leftover { point: String },
    /// A mounted `/dev/…` device without a device node.
    #[error(
        "{device} is mounted at {point} but has no device node, so no disk is known to be unused"
    )]
    NoNode { device: String, point: String },
    /// The whole disks below a mounted device cannot be found.
    #[error("cannot find the disk of {device}: {error}")]
    MountDisks {
        device: String,
        #[source]
        error: io::Error,
    },
    /// A target lies on `disk`, which holds a mounted device.
    #[error(
        "/dev/{disk} is in use by the running system ({device} is mounted at {point}), so nothing is written to it"
    )]
    InUse {
        disk: String,
        device: String,
        point: String,
    },
}
