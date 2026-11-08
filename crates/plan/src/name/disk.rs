//! The name rules of the Boot repair page. Every device ends up as an
//! argument of `mount`, `cryptsetup` or `grub-install` running as root, so
//! a device is never a free path: it is one of the kernel's own names for
//! a disk or a partition below `/dev`, or a device-mapper name below
//! `/dev/mapper`. Nothing else can be written.

#[cfg(test)]
mod tests;

use super::rules::{check_characters, check_shape, check_word};
use super::{Name, NameKind, Reason, Rule};

const DEV: &str = "/dev/";

const MAPPER: &str = "/dev/mapper/";

/// The kernel's `DM_NAME_LEN` is 128 with the final NUL.
const MAX_MAPPER_CHARS: usize = 127;

const MAX_BOOTLOADER_ID_CHARS: usize = 32;

/// Device-mapper's own control node, which is no mapped device.
const MAPPER_CONTROL: &str = "control";

/// Longer disk letters or numbers than this name no disk a person owns.
const MAX_INDEX_CHARS: usize = 3;

/// A whole disk: `/dev/sda`, `/dev/vdb`, `/dev/nvme0n1`, `/dev/mmcblk0`.
pub type DiskDevice = Name<DiskRule>;
/// A partition: `/dev/sda2`, `/dev/nvme0n1p2`, `/dev/mmcblk0p1`.
pub type PartitionDevice = Name<PartitionRule>;
/// What a file system lies on: a partition or `/dev/mapper/<name>`.
pub type VolumeDevice = Name<VolumeRule>;
/// The folder below `EFI/` on the EFI system partition and the name the
/// firmware's boot menu shows.
pub type BootloaderId = Name<BootloaderIdRule>;

pub struct DiskRule;
pub struct PartitionRule;
pub struct VolumeRule;
pub struct MapperRule;
pub struct BootloaderIdRule;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Disk,
    Partition,
}

impl Rule for DiskRule {
    const KIND: NameKind = NameKind::Disk;

    fn check(value: &str) -> Result<(), Reason> {
        check_kernel_device(value, Shape::Disk)
    }
}

impl Rule for PartitionRule {
    const KIND: NameKind = NameKind::Partition;

    fn check(value: &str) -> Result<(), Reason> {
        check_kernel_device(value, Shape::Partition)
    }
}

impl Rule for VolumeRule {
    const KIND: NameKind = NameKind::Volume;

    fn check(value: &str) -> Result<(), Reason> {
        value.strip_prefix(MAPPER).map_or_else(
            || check_kernel_device(value, Shape::Partition),
            MapperRule::check,
        )
    }
}

/// Letters, digits and `._+-` after a letter or digit: one path component
/// that is no option, no `.` and no `..`. LVM writes `vg-root`, cryptsetup
/// `luks-<uuid>`.
impl Rule for MapperRule {
    const KIND: NameKind = NameKind::Mapper;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_MAPPER_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_word(value)?;

        if value == MAPPER_CONTROL {
            Err(Reason::UnknownDevice)
        } else {
            Ok(())
        }
    }
}

/// `grub-install` makes a folder of it and hands it to `efibootmgr` as a
/// label. Letters, digits, `_` and `-` after a letter or digit keep it one
/// folder name that is no option.
impl Rule for BootloaderIdRule {
    const KIND: NameKind = NameKind::BootloaderId;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_BOOTLOADER_ID_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_characters(value, |character| {
            character.is_ascii_alphanumeric() || "_-".contains(character)
        })
    }
}

impl VolumeDevice {
    /// Whether this is a name below `/dev/mapper`, which is a symlink to
    /// the device node and not the node itself.
    pub fn is_mapped(&self) -> bool {
        self.value.starts_with(MAPPER)
    }
}

impl From<PartitionDevice> for VolumeDevice {
    fn from(partition: PartitionDevice) -> Self {
        Self {
            value: partition.value,
            rule: std::marker::PhantomData,
        }
    }
}

fn check_kernel_device(value: &str, wanted: Shape) -> Result<(), Reason> {
    let Some(name) = value.strip_prefix(DEV) else {
        return Err(if value.is_empty() {
            Reason::Empty
        } else {
            Reason::UnknownDevice
        });
    };

    if shape(name) == Some(wanted) {
        Ok(())
    } else {
        Err(Reason::UnknownDevice)
    }
}

/// `sda`, `vdb3`, `nvme0n1`, `nvme0n1p2`, `mmcblk0`, `mmcblk0p1`: the
/// names of the SCSI, virtio, `NVMe` and MMC drivers. Anything else, such
/// as `loop0`, `sr0`, `zram0`, `dm-0` or a name with a `/`, is none.
fn shape(name: &str) -> Option<Shape> {
    if let Some(rest) = name.strip_prefix("sd").or_else(|| name.strip_prefix("vd")) {
        return lettered(rest);
    }

    if let Some(rest) = name.strip_prefix("nvme") {
        let (_controller, rest) = number(rest)?;
        let (_namespace, rest) = number(rest.strip_prefix('n')?)?;
        return numbered(rest);
    }

    let (_card, rest) = number(name.strip_prefix("mmcblk")?)?;

    numbered(rest)
}

/// `a` to `zzz`, then nothing or a partition number.
fn lettered(rest: &str) -> Option<Shape> {
    let letters = rest.chars().take_while(char::is_ascii_lowercase).count();

    if !(1..=MAX_INDEX_CHARS).contains(&letters) {
        return None;
    }

    match rest.get(letters..)? {
        "" => Some(Shape::Disk),
        partition => partition_number(partition).then_some(Shape::Partition),
    }
}

/// Nothing, or `p` and a partition number.
fn numbered(rest: &str) -> Option<Shape> {
    match rest.strip_prefix('p') {
        None if rest.is_empty() => Some(Shape::Disk),
        None => None,
        Some(partition) => partition_number(partition).then_some(Shape::Partition),
    }
}

/// A number without a leading zero, and what follows it.
fn number(text: &str) -> Option<(&str, &str)> {
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    let (number, rest) = text.split_at_checked(digits)?;

    let plain =
        (1..=MAX_INDEX_CHARS).contains(&digits) && (number == "0" || !number.starts_with('0'));

    plain.then_some((number, rest))
}

/// Partitions count from 1.
fn partition_number(text: &str) -> bool {
    number(text).is_some_and(|(number, rest)| rest.is_empty() && number != "0")
}
