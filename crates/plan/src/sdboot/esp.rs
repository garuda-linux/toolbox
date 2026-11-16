#[cfg(test)]
mod tests;

use crate::repair::ESP_FOLDER;

/// Where the EFI system partition is mounted, as `bootctl` looks for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EspPlace {
    Efi,
    Boot,
    BootEfi,
}

impl EspPlace {
    /// In `bootctl`'s order.
    pub const ALL: [Self; 3] = [Self::Efi, Self::Boot, Self::BootEfi];

    /// The folder below `/`.
    pub const fn inside(self) -> &'static str {
        match self {
            Self::Efi => "efi",
            Self::Boot => "boot",
            Self::BootEfi => ESP_FOLDER,
        }
    }

    /// `loader.conf` below `/`. Each place is spelled out, since `secure`
    /// opens fixed paths only.
    pub const fn loader_conf(self) -> &'static str {
        match self {
            Self::Efi => "efi/loader/loader.conf",
            Self::Boot => "boot/loader/loader.conf",
            Self::BootEfi => "boot/efi/loader/loader.conf",
        }
    }
}

pub fn esp_place(mountinfo: &str) -> Option<EspPlace> {
    let fat: Vec<&str> = mountinfo
        .lines()
        .filter_map(|line| {
            let (mount, filesystem) = line.split_once(" - ")?;
            let place = mount.split_whitespace().nth(4)?;
            let kind = filesystem.split_whitespace().next()?;

            (kind == "vfat").then_some(place)
        })
        .collect();

    EspPlace::ALL.into_iter().find(|place| {
        fat.iter()
            .any(|mounted| mounted.strip_prefix('/') == Some(place.inside()))
    })
}
