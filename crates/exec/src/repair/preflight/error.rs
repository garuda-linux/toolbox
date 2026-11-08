//! Why the helper's look at the devices refuses a boot repair. The text is
//! what the Console and the reject event show.

use std::{error, fmt, io};

#[derive(Debug)]
pub enum PreflightError {
    /// A UEFI install without `/sys/firmware/efi`.
    NotUefi,
    NoBlockDevice(String),
    NoWholeDisk(String),
    NoPartition(String),
    NoEsp(String),
    NoLuks(String),
    /// The kernel's record of the device at `path` cannot be read.
    Lookup {
        path: String,
        error: io::Error,
    },
    /// The whole disks below a target cannot be found.
    TargetDisks(io::Error),
    MountTable(io::Error),
    /// A mount at or below the repair's folder, left by an earlier run.
    Leftover {
        point: String,
    },
    /// A mounted `/dev/…` source without a device node.
    NoNode {
        source: String,
        point: String,
    },
    /// The whole disks below a mounted device cannot be found.
    MountDisks {
        source: String,
        error: io::Error,
    },
    /// A target lies on `disk`, which holds a mounted device.
    InUse {
        disk: String,
        source: String,
        point: String,
    },
}

impl fmt::Display for PreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUefi => f.write_str(
                "the live system was not started through UEFI, so GRUB cannot be registered with the firmware",
            ),
            Self::NoBlockDevice(path) => write!(f, "{path} is no block device"),
            Self::NoWholeDisk(path) => write!(f, "{path} is no whole disk"),
            Self::NoPartition(path) => write!(f, "{path} is no partition"),
            Self::NoEsp(path) => write!(f, "{path} is no EFI system partition"),
            Self::NoLuks(path) => write!(f, "{path} is no LUKS partition"),
            Self::Lookup { path, error } => write!(f, "cannot look up {path} in /sys: {error}"),
            Self::TargetDisks(error) => {
                write!(f, "cannot find the disk of a device in /sys: {error}")
            }
            Self::MountTable(error) => write!(f, "cannot read the mount table: {error}"),
            Self::Leftover { point } => write!(
                f,
                "{point} is still mounted from an earlier repair, unmount it or restart the live system"
            ),
            Self::NoNode { source, point } => write!(
                f,
                "{source} is mounted at {point} but has no device node, so no disk is known to be unused"
            ),
            Self::MountDisks { source, error } => {
                write!(f, "cannot find the disk of {source}: {error}")
            }
            Self::InUse {
                disk,
                source,
                point,
            } => write!(
                f,
                "/dev/{disk} is in use by the running system ({source} is mounted at {point}), so nothing is written to it"
            ),
        }
    }
}

impl error::Error for PreflightError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Lookup { path: _, error }
            | Self::MountDisks { source: _, error }
            | Self::TargetDisks(error)
            | Self::MountTable(error) => Some(error),
            Self::NotUefi
            | Self::NoBlockDevice(_)
            | Self::NoWholeDisk(_)
            | Self::NoPartition(_)
            | Self::NoEsp(_)
            | Self::NoLuks(_)
            | Self::Leftover { point: _ }
            | Self::NoNode {
                source: _,
                point: _,
            }
            | Self::InUse {
                disk: _,
                source: _,
                point: _,
            } => None,
        }
    }
}
