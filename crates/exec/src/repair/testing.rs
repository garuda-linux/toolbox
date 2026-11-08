//! A captured machine in a temporary folder: `/sys`, udev's records and
//! the mount table rebuilt from the files in `fixtures/`, and device
//! nodes as a list, since only root can make a real one.
//!
//! Captures: `lima` is a Garuda Lima VM running from `vda` with LUKS and
//! LVM on `vdb`, `bare-metal` a laptop running from `nvme0n1` with an
//! unused second disk `nvme1n1`, `live` the Garuda live ISO in QEMU with
//! its dm-verity layers on `loop0` and an empty `vda`.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::disks::{DeviceNumber, Nodes};
use super::preflight::System;

pub const ESP_TYPE: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/repair/fixtures");

/// Device-mapper's block major, as every capture shows it.
const DM_MAJOR: u32 = 253;

pub struct Machine {
    dir: TempDir,
    nodes: BTreeMap<PathBuf, DeviceNumber>,
    /// Names below `/dev/mapper`, which are symlinks.
    links: BTreeMap<PathBuf, DeviceNumber>,
    folders: BTreeMap<String, PathBuf>,
    mounts: String,
}

impl Nodes for Machine {
    fn block_device(&self, path: &Path, follow: bool) -> Option<DeviceNumber> {
        self.nodes
            .get(path)
            .copied()
            .or_else(|| follow.then(|| self.links.get(path).copied()).flatten())
    }
}

impl Machine {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();

        for folder in ["sys/dev/block", "sys/devices", "run/udev/data", "proc/self"] {
            fs::create_dir_all(dir.path().join(folder)).unwrap();
        }

        let machine = Self {
            dir,
            nodes: BTreeMap::new(),
            links: BTreeMap::new(),
            folders: BTreeMap::new(),
            mounts: String::new(),
        };

        machine.write_mounts();
        machine
    }

    pub fn lima() -> Self {
        Self::capture("", "udev/run/udev/data")
    }

    pub fn bare_metal() -> Self {
        Self::capture("bare-metal", "udev")
    }

    pub fn live() -> Self {
        Self::capture("live", "udev")
    }

    fn capture(name: &str, udev: &str) -> Self {
        let mut machine = Self::new();
        let source = Path::new(FIXTURES).join(name);
        let read = |file: &str| fs::read_to_string(source.join(file)).unwrap();

        for line in read("sys-dev-block").lines() {
            let (number, link) = line.split_once(' ').unwrap();
            let folder = machine
                .root()
                .join("sys")
                .join(link.strip_prefix("../../").unwrap());

            fs::create_dir_all(&folder).unwrap();
            symlink(&folder, machine.root().join("sys/dev/block").join(number)).unwrap();
            machine.folders.insert(
                folder.file_name().unwrap().to_str().unwrap().to_owned(),
                folder,
            );
        }

        for line in read("sys-class-block").lines() {
            machine.class_entry(line);
        }

        for record in fs::read_dir(source.join(udev)).unwrap() {
            machine.udev_record(&record.unwrap().path());
        }

        machine.mounts = read("mountinfo");
        machine.write_mounts();

        // The capture has no listing of /sys/firmware, but efivarfs only
        // mounts when the firmware started the system through UEFI.
        if machine.mounts.contains(" /sys/firmware/efi/efivars ") {
            fs::create_dir_all(machine.root().join("sys/firmware/efi")).unwrap();
        }

        machine
    }

    /// `dm-0 dev=253:0 partition= slaves=vdb `
    fn class_entry(&mut self, line: &str) {
        let (name, rest) = line.split_once(" dev=").unwrap();
        let (number, rest) = rest.split_once(" partition=").unwrap();
        let (partition, slaves) = rest.split_once(" slaves=").unwrap();
        let folder = self.folders[name].clone();

        if !partition.is_empty() {
            fs::write(folder.join("partition"), format!("{partition}\n")).unwrap();
        }

        for slave in slaves.split_whitespace() {
            fs::create_dir_all(folder.join("slaves")).unwrap();
            symlink(&self.folders[slave], folder.join("slaves").join(slave)).unwrap();
        }

        self.nodes.insert(node(name), device_number(number));
    }

    fn udev_record(&mut self, path: &Path) {
        let file = path.file_name().unwrap().to_str().unwrap();
        let number = device_number(file.strip_prefix('b').unwrap());
        let record = fs::read_to_string(path).unwrap();

        for mapped in record
            .lines()
            .filter_map(|line| line.strip_prefix("S:mapper/"))
        {
            self.links
                .insert(PathBuf::from(format!("/dev/mapper/{mapped}")), number);
        }

        fs::write(self.root().join("run/udev/data").join(file), record).unwrap();
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn system(&self) -> System<'_> {
        System {
            root: self.dir.path(),
            nodes: self,
        }
    }

    pub fn number(&self, name: &str) -> DeviceNumber {
        self.nodes[&node(name)]
    }

    pub fn edit_record(&self, name: &str, from: &str, to: &str) {
        let number = self.number(name);
        let path = self
            .root()
            .join(format!("run/udev/data/b{}:{}", number.major, number.minor));

        let record = fs::read_to_string(&path).unwrap();

        assert!(record.contains(from), "{from} is not in {}", path.display());
        fs::write(&path, record.replacen(from, to, 1)).unwrap();
    }

    /// Adds a mount written like the captured one at `like`, with another
    /// source and mount point.
    pub fn mount_as(&mut self, like: &str, source: &str, point: &str) {
        let captured = self
            .mounts
            .lines()
            .find(|line| line.split(' ').nth(4) == Some(like))
            .unwrap()
            .to_owned();

        let (before, after) = captured.split_once(" - ").unwrap();
        let mut before: Vec<&str> = before.split(' ').collect();
        let mut after: Vec<&str> = after.split(' ').collect();
        let point = point.replace(' ', "\\040");

        *before.get_mut(4).unwrap() = &point;
        *after.get_mut(1).unwrap() = source;

        for piece in [&before.join(" "), " - ", &after.join(" "), "\n"] {
            self.mounts.push_str(piece);
        }

        self.write_mounts();
    }

    pub fn unmount_from(&mut self, prefix: &str) {
        self.mounts = self
            .mounts
            .lines()
            .filter(|line| {
                let source = line
                    .split_once(" - ")
                    .and_then(|(_, after)| after.split(' ').nth(1));

                !source.is_some_and(|source| source.starts_with(prefix))
            })
            .flat_map(|line| [line, "\n"])
            .collect();
        self.write_mounts();
    }

    /// Removes the entry of `/dev/<name>` from `/sys/dev/block`, as when
    /// the kernel loses the device while its node stays.
    pub fn lose(&self, name: &str) {
        let number = self.number(name);

        fs::remove_file(
            self.root()
                .join(format!("sys/dev/block/{}:{}", number.major, number.minor)),
        )
        .unwrap();
    }

    /// A device-mapper device `dm-<minor>` on `below`, for stacks no
    /// capture has.
    pub fn mapped(&mut self, minor: u32, below: &str) {
        let name = format!("dm-{minor}");
        let folder = self.root().join("sys/devices/virtual/block").join(&name);

        fs::create_dir_all(folder.join("slaves")).unwrap();
        symlink(&self.folders[below], folder.join("slaves").join(below)).unwrap();
        symlink(
            &folder,
            self.root()
                .join(format!("sys/dev/block/{DM_MAJOR}:{minor}")),
        )
        .unwrap();
        self.folders.insert(name.clone(), folder);
        self.nodes.insert(
            node(&name),
            DeviceNumber {
                major: DM_MAJOR,
                minor,
            },
        );
    }

    fn write_mounts(&self) {
        fs::write(self.dir.path().join("proc/self/mountinfo"), &self.mounts).unwrap();
    }
}

fn node(name: &str) -> PathBuf {
    PathBuf::from(format!("/dev/{name}"))
}

fn device_number(text: &str) -> DeviceNumber {
    let (major, minor) = text.split_once(':').unwrap();

    DeviceNumber {
        major: major.parse().unwrap(),
        minor: minor.parse().unwrap(),
    }
}
