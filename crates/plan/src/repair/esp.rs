//! A repair mounts the EFI system partition where the installed system's
//! own `/etc/fstab` does, so bootctl and kernel-install find it in the
//! place the system expects. Only these three places are taken.

#[cfg(test)]
mod tests;

use super::ESP_FOLDER;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EspPlace {
    Efi,
    Boot,
    BootEfi,
}

/// What the mounted system starts with, seen from its own files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstalledLoader {
    Grub,
    /// `None` when its fstab mounts no FAT file system at a known place.
    SystemdBoot {
        esp: Option<EspPlace>,
    },
}

impl EspPlace {
    pub const fn inside(self) -> &'static str {
        match self {
            Self::Efi => "efi",
            Self::Boot => "boot",
            Self::BootEfi => ESP_FOLDER,
        }
    }

    pub fn from_fstab(fstab: &str) -> Option<Self> {
        fstab
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .filter_map(|line| {
                let mut fields = line.split_whitespace().skip(1);
                let point = fields.next()?;

                (fields.next()? == "vfat").then_some(point)
            })
            .find_map(|point| match point {
                "/efi" => Some(Self::Efi),
                "/boot" => Some(Self::Boot),
                "/boot/efi" => Some(Self::BootEfi),
                _ => None,
            })
    }
}

impl InstalledLoader {
    /// `install_conf` is the system's `/etc/kernel/install.conf`.
    pub fn of(install_conf: Option<&str>, fstab: Option<&str>) -> Self {
        if install_conf.is_some_and(crate::sdboot::is_uki_layout) {
            Self::SystemdBoot {
                esp: fstab.and_then(EspPlace::from_fstab),
            }
        } else {
            Self::Grub
        }
    }
}
