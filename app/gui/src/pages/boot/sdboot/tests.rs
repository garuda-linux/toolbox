use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::{BootLoader, KernelCmdline};
use toolbox_plan::KernelParameters;
use toolbox_plan::sdboot::{LoaderEntry, LoaderSettings, SdBootMenu, sections};
use toolbox_system::{Host, Menu, MenuRead};

use super::super::tests::{Shown, page};
use crate::boot::Boot;

/// Files of the Garuda Lima image.
const LOADER_CONF: &str =
    include_str!("../../../../../../crates/plan/src/sdboot/fixtures/loader.conf");
const KERNEL_CMDLINE: &str =
    include_str!("../../../../../../crates/state/src/boot/fixtures/kernel-cmdline");
const UKI: &[u8] = include_bytes!(
    "../../../../../../crates/plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head"
);
const FALLBACK: &[u8] = include_bytes!(
    "../../../../../../crates/plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head"
);

fn os_release(image: &[u8]) -> String {
    let osrel = sections(image)
        .into_iter()
        .find(|section| section.name == ".osrel")
        .unwrap();

    let start = usize::try_from(osrel.offset).unwrap();
    let end = start + usize::try_from(osrel.size).unwrap();

    String::from_utf8(image[start..end].to_vec()).unwrap()
}

fn image_menu(conf: &str) -> Menu {
    Menu::Read(MenuRead::SdBoot(SdBootMenu {
        settings: LoaderSettings::parse(conf),
        entries: vec![
            LoaderEntry::uki(
                "garuda-7.3.1-1-garuda-fallback.efi",
                &os_release(FALLBACK),
                None,
            )
            .unwrap(),
            LoaderEntry::uki("garuda-7.3.1-1-garuda.efi", &os_release(UKI), None).unwrap(),
        ],
    }))
}

fn image() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Host::default());
    snapshot.insert(BootLoader::SystemdBoot);
    snapshot.insert(KernelCmdline(
        KernelParameters::parse(KERNEL_CMDLINE).unwrap(),
    ));
    snapshot
}

fn shown(menu: Menu) -> Shown {
    page(image(), Boot::showing(menu, None, None))
}

#[test]
fn the_images_settings_are_systemd_boots_defaults() {
    let harness = shown(image_menu(LOADER_CONF));

    assert!(
        harness
            .query_by_label_contains("The first entry of the menu")
            .is_some()
    );

    // Timeout, editor, automatic entries and the firmware entry.
    assert_eq!(
        harness
            .query_all_by_label_contains("systemd-boot's default")
            .count(),
        4
    );
    assert!(
        harness
            .query_by_label("GRUB is not the boot loader of this system")
            .is_none()
    );
}

#[test]
fn the_images_command_line_is_shown_whole() {
    let harness = shown(image_menu(LOADER_CONF));

    assert!(
        harness
            .query_by_label_contains("rootflags=subvol=@ rw loglevel=3 quiet")
            .is_some()
    );
}

/// The image's `loader.conf` with its commented timeout switched on, and
/// its UKI as the default.
#[test]
fn a_set_timeout_and_default_show_with_the_entrys_title() {
    let conf = format!(
        "{}default garuda-7.3.1-1-garuda.efi\n",
        LOADER_CONF.replace("#timeout", "timeout")
    );

    let harness = shown(image_menu(&conf));

    assert!(harness.query_by_label_contains("3 s").is_some());
    assert!(harness.query_by_label_contains("Garuda Linux").is_some());
}

#[test]
fn the_menu_is_read_when_the_page_shows() {
    let harness = shown(Menu::Loading);

    assert!(
        harness
            .query_by_label_contains("Reading the boot menu")
            .is_some()
    );
}

/// The command line is known without the menu, so a failed read leaves it
/// on the page.
#[test]
fn a_menu_that_could_not_be_read_says_why_and_keeps_the_command_line() {
    let harness = shown(Menu::Read(MenuRead::Unknown(
        "no ESP is mounted".to_owned(),
    )));

    assert!(
        harness
            .query_by_label_contains("The boot menu could not be read")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("rootflags=subvol=@")
            .is_some()
    );
}

/// The image's menu on a system without `/etc/kernel/cmdline`, as NixOS
/// boots: its entries carry their own options.
#[test]
fn without_kernel_cmdline_the_page_says_so_instead_of_none() {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Host::default());
    snapshot.insert(BootLoader::SystemdBoot);

    let harness = page(snapshot, Boot::showing(image_menu(LOADER_CONF), None, None));

    assert!(
        harness
            .query_by_label_contains("This system has no /etc/kernel/cmdline")
            .is_some()
    );
}
