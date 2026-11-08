//! A made-up machine in a temporary folder: `/sys`, udev's records and the
//! mount table as files, and device nodes as a list, since only root can
//! make a real one.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::disks::{DeviceNumber, Nodes};
use super::preflight::System;

pub const ESP_TYPE: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
const LINUX_TYPE: &str = "0fc63daf-8483-4772-8e79-3d69d8477de4";

pub const ESP: [(&str, &str); 2] = [("ID_FS_TYPE", "vfat"), ("ID_PART_ENTRY_TYPE", ESP_TYPE)];

pub struct Machine {
    dir: TempDir,
    nodes: BTreeMap<PathBuf, DeviceNumber>,
    /// Names below `/dev/mapper`, which are symlinks.
    links: BTreeMap<PathBuf, DeviceNumber>,
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

        fs::create_dir_all(dir.path().join("sys/dev/block")).unwrap();
        fs::create_dir_all(dir.path().join("sys/devices")).unwrap();
        fs::create_dir_all(dir.path().join("run/udev/data")).unwrap();
        fs::create_dir_all(dir.path().join("proc/self")).unwrap();

        let machine = Self {
            dir,
            nodes: BTreeMap::new(),
            links: BTreeMap::new(),
            mounts: String::new(),
        };

        machine.write_mounts();
        machine
    }

    /// A UEFI machine with one disk as Garuda installs it, started from
    /// a USB stick: `sda1` the EFI partition, `sda2` btrfs, `sda3` LUKS,
    /// and `sdb1` the mounted live medium.
    pub fn live() -> Self {
        let mut machine = Self::new();

        machine.efi();
        machine.disk("sda", 8, 0);
        machine.partition("sda", "sda1", (8, 1), &ESP);
        machine.partition(
            "sda",
            "sda2",
            (8, 2),
            &[("ID_FS_TYPE", "btrfs"), ("ID_PART_ENTRY_TYPE", LINUX_TYPE)],
        );
        machine.partition(
            "sda",
            "sda3",
            (8, 3),
            &[
                ("ID_FS_TYPE", "crypto_LUKS"),
                ("ID_PART_ENTRY_TYPE", LINUX_TYPE),
            ],
        );
        machine.disk("sdb", 8, 16);
        machine.partition("sdb", "sdb1", (8, 17), &[("ID_FS_TYPE", "iso9660")]);
        machine.mount("/dev/sdb1", "/run/archiso/bootmnt");
        machine.mount("overlay", "/");
        machine
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

    pub fn efi(&self) {
        fs::create_dir_all(self.dir.path().join("sys/firmware/efi")).unwrap();
    }

    pub fn disk(&mut self, name: &str, major: u32, minor: u32) {
        let folder = self.dir.path().join("sys/devices/block").join(name);

        fs::create_dir_all(&folder).unwrap();
        self.register(name, &folder, (major, minor));
    }

    pub fn partition(
        &mut self,
        disk: &str,
        name: &str,
        number: (u32, u32),
        properties: &[(&str, &str)],
    ) {
        let folder = self
            .dir
            .path()
            .join("sys/devices/block")
            .join(disk)
            .join(name);

        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("partition"), "1\n").unwrap();
        self.register(name, &folder, number);

        let mut record = String::from("S:disk/by-partuuid/x\n");

        for (key, value) in properties {
            writeln!(record, "E:{key}={value}").unwrap();
        }

        fs::write(
            self.dir
                .path()
                .join(format!("run/udev/data/b{}:{}", number.0, number.1)),
            record,
        )
        .unwrap();
    }

    /// A device-mapper device `dm-<minor>` on `below` (kernel names),
    /// with the name `name` in `/dev/mapper`.
    pub fn mapped(&mut self, name: &str, minor: u32, below: &[&str]) {
        let kernel_name = format!("dm-{minor}");
        let folder = self
            .dir
            .path()
            .join("sys/devices/virtual/block")
            .join(&kernel_name);

        fs::create_dir_all(folder.join("slaves")).unwrap();

        for slave in below {
            let target = self
                .dir
                .path()
                .join("sys/dev/names")
                .join(slave)
                .canonicalize()
                .unwrap();

            symlink(target, folder.join("slaves").join(slave)).unwrap();
        }

        self.register(&kernel_name, &folder, (254, minor));
        self.links.insert(
            PathBuf::from(format!("/dev/mapper/{name}")),
            DeviceNumber { major: 254, minor },
        );
    }

    pub fn mount(&mut self, source: &str, point: &str) {
        let id = self.mounts.lines().count() + 20;
        let point = point.replace(' ', "\\040");

        writeln!(
            self.mounts,
            "{id} 1 0:{id} / {point} rw,relatime shared:{id} - auto {source} rw"
        )
        .unwrap();
        self.write_mounts();
    }

    fn write_mounts(&self) {
        fs::write(self.dir.path().join("proc/self/mountinfo"), &self.mounts).unwrap();
    }

    fn register(&mut self, name: &str, folder: &Path, number: (u32, u32)) {
        let block = self.dir.path().join("sys/dev/block");
        let names = self.dir.path().join("sys/dev/names");

        fs::create_dir_all(&names).unwrap();
        symlink(folder, block.join(format!("{}:{}", number.0, number.1))).unwrap();
        symlink(folder, names.join(name)).unwrap();
        self.nodes.insert(
            PathBuf::from(format!("/dev/{name}")),
            DeviceNumber {
                major: number.0,
                minor: number.1,
            },
        );
    }
}
