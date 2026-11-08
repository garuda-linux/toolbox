//! Reading `lsblk --json --tree`: the disks, their EFI system partitions
//! and the partitions that could hold an installed system.
//!
//! Only kernel names of the shapes `toolbox-plan` allows survive, so
//! nothing else can be picked on the page.

#[cfg(test)]
mod tests;

use serde::Deserialize;
use toolbox_plan::{DiskDevice, PartitionDevice, RepairRoot, VolumeDevice};

use super::{Disk, Encryption, Esp, Inventory, System};

/// What `lsblk` is asked for.
///
/// `--tree` nests partitions and mapped devices below their disk, which
/// the flat list of `--output` alone does not. `--bytes`, because its own
/// short sizes (`465.8G`) are in English and the page words sizes in the
/// reader's language.
pub const LSBLK_ARGUMENTS: [&str; 5] = [
    "--json",
    "--bytes",
    "--tree",
    "--output",
    "PATH,TYPE,SIZE,FSTYPE,PARTTYPE,LABEL,MODEL,MOUNTPOINTS",
];

/// The partition type of an EFI system partition on a GPT disk, and its
/// counterpart in an MBR partition table.
const ESP_TYPES: [&str; 2] = ["c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "0xef"];

const ESP_FILE_SYSTEM: &str = "vfat";
const LUKS: &str = "crypto_LUKS";

/// The file systems an Arch based system is installed on.
const ROOT_FILE_SYSTEMS: [&str; 7] = ["btrfs", "ext4", "ext3", "ext2", "xfs", "f2fs", "jfs"];

/// How `lsblk` marks active swap in the mount point column. Swap does
/// not keep a disk from getting a boot loader.
const SWAP_MARK: &str = "[SWAP]";

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("lsblk printed no device list: {0}")]
pub struct InvalidListing(String);

#[derive(Deserialize)]
struct Listing {
    blockdevices: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    path: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default, deserialize_with = "bytes")]
    size: Option<u64>,
    fstype: Option<String>,
    parttype: Option<String>,
    label: Option<String>,
    model: Option<String>,
    #[serde(default)]
    mountpoints: Vec<Option<String>>,
    #[serde(default)]
    children: Vec<Self>,
}

impl Node {
    fn text(field: Option<&String>) -> String {
        field.map(|text| text.trim().to_owned()).unwrap_or_default()
    }

    fn file_system(&self) -> &str {
        self.fstype.as_deref().unwrap_or_default()
    }

    fn is_mounted(&self) -> bool {
        self.mountpoints
            .iter()
            .flatten()
            .any(|point| point != SWAP_MARK)
            || self.children.iter().any(Self::is_mounted)
    }

    fn is_esp(&self) -> bool {
        let partition_type = self.parttype.as_deref().unwrap_or_default().to_lowercase();

        ESP_TYPES.contains(&partition_type.as_str()) && self.file_system() == ESP_FILE_SYSTEM
    }
}

struct Place<'disk> {
    disk: &'disk DiskDevice,
    in_use: bool,
    /// An unlocked LUKS device lies between here and the disk.
    encrypted: bool,
}

/// Reads the output of `lsblk` with [`LSBLK_ARGUMENTS`].
///
/// # Errors
///
/// [`InvalidListing`] when the text is no JSON device list.
pub fn parse_lsblk(json: &str) -> Result<Inventory, InvalidListing> {
    let listing: Listing =
        serde_json::from_str(json.trim()).map_err(|error| InvalidListing(error.to_string()))?;

    let mut inventory = Inventory::default();

    for node in &listing.blockdevices {
        let disk = node
            .path
            .as_deref()
            .filter(|_path| node.kind.as_deref() == Some("disk"))
            // A card reader without a card.
            .filter(|_path| node.size != Some(0))
            .and_then(|path| DiskDevice::new(path).ok());

        let Some(disk) = disk else {
            continue;
        };

        let in_use = node.is_mounted();

        inventory.disks.push(Disk {
            device: disk.clone(),
            model: Node::text(node.model.as_ref()),
            size: node.size.unwrap_or_default(),
            in_use,
        });

        let place = Place {
            disk: &disk,
            in_use,
            encrypted: false,
        };

        for child in &node.children {
            visit(child, &place, &mut inventory);
        }
    }

    Ok(inventory)
}

fn visit(node: &Node, place: &Place<'_>, inventory: &mut Inventory) {
    let path = node.path.as_deref().unwrap_or_default();
    let partition = (node.kind.as_deref() == Some("part"))
        .then(|| PartitionDevice::new(path).ok())
        .flatten();

    let below = Place {
        disk: place.disk,
        in_use: place.in_use,
        encrypted: place.encrypted || node.file_system() == LUKS,
    };

    if let Some(partition) = &partition
        && node.is_esp()
    {
        inventory.esps.push(Esp {
            device: partition.clone(),
            label: Node::text(node.label.as_ref()),
            size: node.size.unwrap_or_default(),
            disk: place.disk.clone(),
            in_use: place.in_use,
        });
    }

    let root = if ROOT_FILE_SYSTEMS.contains(&node.file_system()) {
        partition
            .map(VolumeDevice::from)
            .or_else(|| VolumeDevice::new(path).ok().filter(VolumeDevice::is_mapped))
            .map(|device| RepairRoot::Open { device })
    } else if node.file_system() == LUKS && node.children.is_empty() {
        partition.map(|partition| RepairRoot::Locked { partition })
    } else {
        None
    };

    if let Some(root) = root {
        add_system(node, root, place, inventory);
    }

    for child in &node.children {
        visit(child, &below, inventory);
    }
}

/// A volume group over several disks shows the same volume below each
/// of them: one system then, on all those disks.
fn add_system(node: &Node, root: RepairRoot, place: &Place<'_>, inventory: &mut Inventory) {
    if let Some(known) = inventory
        .systems
        .iter_mut()
        .find(|system| system.root == root)
    {
        if !known.disks.contains(place.disk) {
            known.disks.push(place.disk.clone());
        }

        known.in_use |= place.in_use;
        return;
    }

    let encryption = match &root {
        RepairRoot::Locked { partition: _ } => Encryption::Locked,
        RepairRoot::Open { device: _ } if place.encrypted => Encryption::Unlocked,
        RepairRoot::Open { device: _ } => Encryption::None,
    };

    inventory.systems.push(System {
        root,
        label: Node::text(node.label.as_ref()),
        size: node.size.unwrap_or_default(),
        file_system: match encryption {
            Encryption::Locked => String::new(),
            Encryption::None | Encryption::Unlocked => node.file_system().to_owned(),
        },
        encryption,
        disks: vec![place.disk.clone()],
        in_use: place.in_use,
    });
}

/// A size in bytes, which `lsblk --bytes` prints as a number and some
/// versions as a quoted number. Any other shape is an unknown size
/// rather than a listing that cannot be read.
fn bytes<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Size {
        Number(u64),
        Text(String),
        Missing,
    }

    Ok(match Size::deserialize(deserializer)? {
        Size::Number(size) => Some(size),
        Size::Text(text) => text.parse().ok(),
        Size::Missing => None,
    })
}
