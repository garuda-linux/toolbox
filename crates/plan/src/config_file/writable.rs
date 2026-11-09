//! The root-owned files the helper replaces: exactly the root-owned
//! entries of `data/catalog/config-files.toml` without `read_only`. The
//! `read_only` entries (accounts, sudoers, PAM, cron, system units, udev
//! rules, pacman hooks) hand out root by themselves, so they are shown and
//! never written. A fetch takes what the catalog names and nothing else,
//! as `/etc` holds secrets no deny list can know in advance.
//!
//! The tables are written out here because the helper links no TOML.
//! `toolbox-catalog` refuses a writable root-owned entry missing from them
//! and a read-only one they cover, and `cargo xtask catalog-validate` fails
//! for a line no entry needs.

#[cfg(test)]
mod tests;

use super::ConfigFilePath;
use super::readable::{READABLE_CONFIG_FILES, READABLE_CONFIG_FOLDERS};

/// The root-owned files of the catalog, sorted.
pub const WRITABLE_CONFIG_FILES: [&str; 51] = [
    "/boot/grub/grub.cfg",
    "/etc/NetworkManager/NetworkManager.conf",
    "/etc/X11/xorg.conf",
    "/etc/X11/xorg.conf.d/10-monitor.conf",
    "/etc/X11/xorg.conf.d/20-gpu.conf",
    "/etc/adjtime",
    "/etc/asound.conf",
    "/etc/bash.bashrc",
    "/etc/bluetooth/main.conf",
    "/etc/btrfs-assistant.conf",
    "/etc/default/cpupower",
    "/etc/default/grub",
    "/etc/environment",
    "/etc/fonts/local.conf",
    "/etc/fstab",
    "/etc/gamemode.ini",
    "/etc/garuda/garuda-update/config",
    "/etc/gdm/custom.conf",
    "/etc/hostname",
    "/etc/hosts",
    "/etc/hosts.allow",
    "/etc/hosts.deny",
    "/etc/kernel/cmdline",
    "/etc/lightdm/lightdm.conf",
    "/etc/locale.conf",
    "/etc/locale.gen",
    "/etc/makepkg.conf",
    "/etc/mkinitcpio.conf",
    "/etc/modprobe.d/alsa-base.conf",
    "/etc/modprobe.d/amdgpu.conf",
    "/etc/modprobe.d/blacklist-nouveau.conf",
    "/etc/modprobe.d/nvidia.conf",
    "/etc/nftables.conf",
    "/etc/os-release",
    "/etc/pacman.conf",
    "/etc/pacman.d/chaotic-mirrorlist",
    "/etc/pacman.d/mirrorlist",
    "/etc/plymouth/plymouthd.conf",
    "/etc/resolv.conf",
    "/etc/rsyslog.conf",
    "/etc/snapper/configs/root",
    "/etc/ssh/sshd_config",
    "/etc/sysctl.conf",
    "/etc/systemd/journald.conf",
    "/etc/systemd/logind.conf",
    "/etc/systemd/system.conf",
    "/etc/systemd/user.conf",
    "/etc/systemd/zram-generator.conf",
    "/etc/vconsole.conf",
    "/usr/share/grub/themes/garuda-dr460nized/theme.txt",
    "/usr/share/icons/default/index.theme",
];

/// The root-owned folders of the catalog (`directory = true`), sorted.
/// The page lists their regular files one level deep, so a write may name
/// any file directly in one of them and nothing further down.
pub const WRITABLE_CONFIG_FOLDERS: [&str; 5] = [
    "/etc/X11/xorg.conf.d",
    "/etc/modprobe.d",
    "/etc/sddm.conf.d",
    "/etc/systemd/network",
    "/etc/systemd/user",
];

impl ConfigFilePath {
    /// Whether the helper may replace this file: a file of the catalog, or
    /// a file directly in one of its folders.
    pub fn is_writable(&self) -> bool {
        listed(self, &WRITABLE_CONFIG_FILES, &WRITABLE_CONFIG_FOLDERS)
    }

    /// Whether the helper may hand this file out: a file it may replace,
    /// or one the catalog only shows. The withheld names of
    /// [`ConfigFilePath`] stay out of both.
    pub fn is_readable(&self) -> bool {
        self.is_writable() || listed(self, &READABLE_CONFIG_FILES, &READABLE_CONFIG_FOLDERS)
    }
}

/// A file of `files`, or a file directly in one of `folders`. Both sorted.
fn listed(path: &ConfigFilePath, files: &[&str], folders: &[&str]) -> bool {
    let text = path.as_str();
    let folder = text.rsplit_once('/').map_or("", |(folder, _)| folder);

    files.binary_search(&text).is_ok() || folders.binary_search(&folder).is_ok()
}
