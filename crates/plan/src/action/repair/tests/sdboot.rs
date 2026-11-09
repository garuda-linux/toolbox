//! A repair of an installed system that starts with systemd-boot, as the
//! helper runs it once it has seen the system's files.

use super::{ROOT, as_run, bios, lines, open, regenerate, repair, uefi};
use crate::action::testing::shown;
use crate::{EspPlace, InstalledLoader, PartitionDevice, RepairWork, RootLayout};

const RECHECK: &str = "check the devices of the repair again";
const INITRAMFS: &str = "chroot /run/garuda-toolbox/boot-repair/root /usr/bin/dracut-rebuild, or chroot /run/garuda-toolbox/boot-repair/root /usr/bin/mkinitcpio -P where dracut-rebuild is not installed";
const UKIS: &str = "SYSTEMD_RELAX_ESP_CHECKS=1 chroot /run/garuda-toolbox/boot-repair/root /usr/lib/garuda/kernel-install-uki add, or SYSTEMD_RELAX_ESP_CHECKS=1 chroot /run/garuda-toolbox/boot-repair/root /usr/bin/kernel-install add-all where kernel-install-uki is not installed";

fn sdboot(place: EspPlace) -> InstalledLoader {
    InstalledLoader::SystemdBoot { esp: Some(place) }
}

fn regenerate_on(esp: &str) -> RepairWork {
    RepairWork::RegenerateConfig {
        esp: Some(PartitionDevice::new(esp).unwrap()),
    }
}

/// The steps between the binds and the cleanup.
fn loader_part(lines: &[String]) -> Vec<String> {
    let first = lines
        .iter()
        .position(|line| line.starts_with("mount --bind /run "))
        .unwrap();

    let last = lines
        .iter()
        .position(|line| line.starts_with("umount "))
        .unwrap();

    lines[first + 1..last].to_vec()
}

#[test]
fn a_reinstall_puts_systemd_boot_where_fstab_mounts_the_esp() {
    let action = repair(
        open("/dev/nvme0n1p2", RootLayout::Subvolume),
        uefi("/dev/nvme0n1p1"),
    );

    let steps = lines(&as_run(&action, sdboot(EspPlace::Efi)).unwrap());

    assert_eq!(
        loader_part(&steps),
        [
            format!("mount -t efivarfs efivarfs {ROOT}/sys/firmware/efi/efivars"),
            format!("mount -o nodev,nosuid /dev/nvme0n1p1 {ROOT}/efi"),
            RECHECK.to_owned(),
            format!(
                "SYSTEMD_RELAX_ESP_CHECKS=1 chroot {ROOT} /usr/bin/bootctl install --esp-path=/efi"
            ),
            INITRAMFS.to_owned(),
            UKIS.to_owned(),
        ]
    );
}

/// The images live on the ESP, so a rebuild mounts it too, but needs no
/// EFI variables: the firmware's entry stays.
#[test]
fn a_rebuild_mounts_the_esp_and_builds_the_images() {
    let action = repair(
        open("/dev/sda2", RootLayout::Plain),
        regenerate_on("/dev/sda1"),
    );

    let steps = lines(&as_run(&action, sdboot(EspPlace::BootEfi)).unwrap());

    assert_eq!(
        loader_part(&steps),
        [
            format!("mount -o nodev,nosuid /dev/sda1 {ROOT}/boot/efi"),
            RECHECK.to_owned(),
            INITRAMFS.to_owned(),
            UKIS.to_owned(),
        ]
    );
}

#[test]
fn what_systemd_boot_cannot_do_is_refused_with_a_reason() {
    let system = open("/dev/sda2", RootLayout::Plain);

    for (work, loader, reason) in [
        (bios("/dev/sda"), sdboot(EspPlace::Efi), "only through UEFI"),
        (regenerate(), sdboot(EspPlace::Efi), "names none"),
        (
            uefi("/dev/sda1"),
            InstalledLoader::SystemdBoot { esp: None },
            "/etc/fstab mounts no EFI system partition",
        ),
    ] {
        let refused = as_run(&repair(system.clone(), work), loader).unwrap_err();

        assert!(refused.contains(reason), "{refused}");
    }
}

/// The boot loader shows only once the system is mounted, so the plan
/// names both ways.
#[test]
fn the_plan_shows_the_steps_of_both_boot_loaders() {
    let action = repair(open("/dev/sda2", RootLayout::Plain), uefi("/dev/sda1"));
    let loader = shown(&action)
        .into_iter()
        .find(|line| line.starts_with("on GRUB: "))
        .unwrap();

    assert!(
        loader.contains("grub-install --target=x86_64-efi"),
        "{loader}"
    );
    assert!(
        loader.contains(", on systemd-boot: mount -t efivarfs"),
        "{loader}"
    );
    assert!(
        loader.contains("bootctl install --esp-path=/efi"),
        "{loader}"
    );
}
