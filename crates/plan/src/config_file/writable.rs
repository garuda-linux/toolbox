//! The root-owned files the helper replaces: exactly the root-owned
//! entries of `data/catalog/config-files.toml` without `read_only`. Reads
//! keep the wider rule of [`ConfigFilePath`], a write is what turns a path
//! into root's code. The `read_only` entries (accounts, sudoers, PAM,
//! cron, system units, udev rules, pacman hooks) hand out root by
//! themselves, so they are shown and never written.
//!
//! The tables are written out here because the helper links no TOML.
//! `toolbox-catalog` refuses a writable root-owned entry missing from them
//! and a read-only one they cover, and `cargo xtask catalog-validate` fails
//! for a line no entry needs.

#[cfg(test)]
mod tests;

use super::ConfigFilePath;

/// The root-owned files of the catalog, sorted.
pub const WRITABLE_CONFIG_FILES: [&str; 50] = [
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
        let text = self.as_str();
        let folder = text.rsplit_once('/').map_or("", |(folder, _)| folder);

        WRITABLE_CONFIG_FILES.binary_search(&text).is_ok()
            || WRITABLE_CONFIG_FOLDERS.binary_search(&folder).is_ok()
    }
}
