//! Packages no page, no command and no helper request removes, each with
//! the reason it stays. Compiled in, so no missing or broken file can
//! empty the list. Kernels are guarded by the Kernels page, not here.

#[cfg(test)]
mod tests;

use crate::PackageName;

pub const PROTECTED: &[(&str, Protection)] = &[
    ("base", Protection::Boot),
    ("filesystem", Protection::Boot),
    ("glibc", Protection::Boot),
    ("linux-firmware", Protection::Boot),
    ("systemd", Protection::Boot),
    ("systemd-libs", Protection::Boot),
    ("systemd-sysvcompat", Protection::Boot),
    ("dbus", Protection::Boot),
    ("dbus-broker", Protection::Boot),
    ("kmod", Protection::Boot),
    ("grub", Protection::Boot),
    ("efibootmgr", Protection::Boot),
    ("mkinitcpio", Protection::Boot),
    ("dracut", Protection::Boot),
    // toolbox-lint: allow(sudo-once) A package name, no program.
    ("sudo", Protection::Root),
    ("polkit", Protection::Root),
    ("shadow", Protection::Login),
    ("pam", Protection::Login),
    ("util-linux", Protection::Login),
    ("bash", Protection::Login),
    ("coreutils", Protection::Login),
    ("networkmanager", Protection::Network),
    ("pacman", Protection::PackageManager),
    ("pacman-mirrorlist", Protection::PackageManager),
    ("archlinux-keyring", Protection::PackageManager),
    ("chaotic-keyring", Protection::PackageManager),
    ("chaotic-mirrorlist", Protection::PackageManager),
    ("garuda-update", Protection::PackageManager),
    ("plasma-desktop", Protection::Desktop),
    ("plasma-workspace", Protection::Desktop),
    ("gnome-shell", Protection::Desktop),
    ("gnome-session", Protection::Desktop),
    ("xfce4-session", Protection::Desktop),
    ("cinnamon", Protection::Desktop),
    ("mate-session-manager", Protection::Desktop),
    ("sddm", Protection::Desktop),
    ("gdm", Protection::Desktop),
    ("lightdm", Protection::Desktop),
];

/// Why no page and no command removes a package.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protection {
    Boot,
    Root,
    Login,
    Network,
    PackageManager,
    Desktop,
}

impl Protection {
    /// The end of its Fluent id, `protected-<reason>`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::Root => "root",
            Self::Login => "login",
            Self::Network => "network",
            Self::PackageManager => "package-manager",
            Self::Desktop => "desktop",
        }
    }
}

/// Why `package` stays installed, `None` when it may go.
pub fn protection(package: &PackageName) -> Option<Protection> {
    PROTECTED
        .iter()
        .find(|(name, _)| *name == package.as_str())
        .map(|(_, why)| *why)
}
