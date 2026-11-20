//! The programs the app, the command line and actions may run, each at a
//! fixed path. A program that is not listed here cannot be started by the
//! helper. The probes and Diagnostics name theirs here too, so no `PATH`
//! decides which program runs.

use std::path::{Path, PathBuf};

/// The programs actions may run, each at a fixed absolute path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Program {
    Bootctl,
    Checkupdates,
    Chfn,
    Chpasswd,
    Chroot,
    Chsh,
    Cryptsetup,
    DbusSend,
    Dkms,
    Dmesg,
    DracutRebuild,
    GarudaChroot,
    GarudaHealth,
    GarudaInxi,
    GarudaUpdate,
    Gpasswd,
    Groupadd,
    Groupdel,
    GrubInstall,
    GrubMkconfig,
    Hblock,
    Journalctl,
    KernelInstall,
    /// Garuda's, from garuda-systemd-boot: it hands kernel-install the
    /// initramfs dracut built.
    KernelInstallUki,
    Less,
    LocaleGen,
    Makepkg,
    Mkinitcpio,
    Modprobe,
    Mount,
    Nmcli,
    Paccache,
    Pacdiff,
    Pacman,
    PacmanKey,
    Paru,
    Ping,
    Pkill,
    PlymouthSetDefaultTheme,
    Snapper,
    Systemctl,
    SystemdAnalyze,
    SystemdDetectVirt,
    Traceroute,
    Umount,
    Useradd,
    Userdel,
    Usermod,
    Visudo,
    // The editors of `crate::Editor`, which only user steps run.
    Emacs,
    Helix,
    Joe,
    Mcedit,
    Micro,
    Nano,
    Ne,
    Neovim,
    Vi,
    Vim,
}

impl Program {
    #[expect(
        clippy::too_many_lines,
        reason = "one arm per program, a new program fails to compile until it names its path"
    )]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Bootctl => "/usr/bin/bootctl",
            Self::Checkupdates => "/usr/bin/checkupdates",
            Self::Chfn => "/usr/bin/chfn",
            Self::Chpasswd => "/usr/bin/chpasswd",
            Self::Chroot => "/usr/bin/chroot",
            Self::Chsh => "/usr/bin/chsh",
            Self::Cryptsetup => "/usr/bin/cryptsetup",
            Self::DbusSend => "/usr/bin/dbus-send",
            Self::Dkms => "/usr/bin/dkms",
            Self::Dmesg => "/usr/bin/dmesg",
            Self::DracutRebuild => "/usr/bin/dracut-rebuild",
            Self::GarudaChroot => "/usr/bin/garuda-chroot",
            Self::GarudaHealth => "/usr/bin/garuda-health",
            Self::GarudaInxi => "/usr/bin/garuda-inxi",
            Self::GarudaUpdate => "/usr/bin/garuda-update",
            Self::Gpasswd => "/usr/bin/gpasswd",
            Self::Groupadd => "/usr/bin/groupadd",
            Self::Groupdel => "/usr/bin/groupdel",
            Self::GrubInstall => "/usr/bin/grub-install",
            Self::GrubMkconfig => "/usr/bin/grub-mkconfig",
            Self::Hblock => "/usr/bin/hblock",
            Self::Journalctl => "/usr/bin/journalctl",
            Self::KernelInstall => "/usr/bin/kernel-install",
            Self::KernelInstallUki => "/usr/lib/garuda/kernel-install-uki",
            Self::Less => "/usr/bin/less",
            Self::LocaleGen => "/usr/bin/locale-gen",
            Self::Makepkg => "/usr/bin/makepkg",
            Self::Mkinitcpio => "/usr/bin/mkinitcpio",
            Self::Modprobe => "/usr/bin/modprobe",
            Self::Mount => "/usr/bin/mount",
            Self::Nmcli => "/usr/bin/nmcli",
            Self::Paccache => "/usr/bin/paccache",
            Self::Pacdiff => "/usr/bin/pacdiff",
            Self::Pacman => "/usr/bin/pacman",
            Self::PacmanKey => "/usr/bin/pacman-key",
            Self::Paru => "/usr/bin/paru",
            Self::Ping => "/usr/bin/ping",
            Self::Pkill => "/usr/bin/pkill",
            Self::PlymouthSetDefaultTheme => "/usr/bin/plymouth-set-default-theme",
            Self::Snapper => "/usr/bin/snapper",
            Self::Systemctl => "/usr/bin/systemctl",
            Self::SystemdAnalyze => "/usr/bin/systemd-analyze",
            Self::SystemdDetectVirt => "/usr/bin/systemd-detect-virt",
            Self::Traceroute => "/usr/bin/traceroute",
            Self::Umount => "/usr/bin/umount",
            Self::Useradd => "/usr/bin/useradd",
            Self::Userdel => "/usr/bin/userdel",
            Self::Usermod => "/usr/bin/usermod",
            Self::Visudo => "/usr/bin/visudo",
            Self::Emacs => "/usr/bin/emacs",
            Self::Helix => "/usr/bin/helix",
            Self::Joe => "/usr/bin/joe",
            Self::Mcedit => "/usr/bin/mcedit",
            Self::Micro => "/usr/bin/micro",
            Self::Nano => "/usr/bin/nano",
            Self::Ne => "/usr/bin/ne",
            Self::Neovim => "/usr/bin/nvim",
            Self::Vi => "/usr/bin/vi",
            Self::Vim => "/usr/bin/vim",
        }
    }

    /// The program's path below `root`, which is `/` outside of tests.
    pub fn below(self, root: &Path) -> PathBuf {
        root.join(self.path().trim_start_matches('/'))
    }

    pub fn name(self) -> &'static str {
        self.path().rsplit('/').next().unwrap_or_default()
    }
}
