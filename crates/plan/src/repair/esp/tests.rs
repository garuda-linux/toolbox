use super::{EspPlace, InstalledLoader};

/// `/etc/fstab` and `/etc/kernel/install.conf` of the Garuda Lima image,
/// and `/etc/fstab` of a Garuda GRUB install (the `garuda-uefi` VM).
const FSTAB: &str = crate::testing::repair::SDBOOT_FSTAB;
const INSTALL_CONF: &str = crate::testing::boot::INSTALL_CONF;
const GRUB_FSTAB: &str = include_str!("../fixtures/grub-fstab");

#[test]
fn the_image_mounts_its_esp_at_efi() {
    assert_eq!(EspPlace::from_fstab(FSTAB), Some(EspPlace::Efi));
}

/// The image's ESP line commented out: no place is guessed.
#[test]
fn a_commented_esp_line_names_no_place() {
    let commented = FSTAB.replace("UUID=6438-BB38", "#UUID=6438-BB38");

    assert_eq!(EspPlace::from_fstab(&commented), None);
}

/// Only FAT counts: the image's btrfs lines name `/` and `/home`.
#[test]
fn the_esp_is_only_a_fat_line() {
    let not_fat = FSTAB.replace("vfat", "ext4");

    assert_eq!(EspPlace::from_fstab(&not_fat), None);
}

#[test]
fn the_image_starts_with_systemd_boot_and_its_esp_at_efi() {
    assert_eq!(
        InstalledLoader::of(Some(INSTALL_CONF), Some(FSTAB)),
        InstalledLoader::SystemdBoot {
            esp: Some(EspPlace::Efi)
        }
    );
}

/// The same fstab without kernel-install's UKI layout is a GRUB system.
#[test]
fn without_the_uki_layout_the_system_starts_with_grub() {
    assert_eq!(
        InstalledLoader::of(None, Some(FSTAB)),
        InstalledLoader::Grub
    );
}

#[test]
fn a_grub_install_mounts_its_esp_at_boot_efi() {
    assert_eq!(EspPlace::from_fstab(GRUB_FSTAB), Some(EspPlace::BootEfi));
    assert_eq!(
        InstalledLoader::of(None, Some(GRUB_FSTAB)),
        InstalledLoader::Grub
    );
}
