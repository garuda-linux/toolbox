use toolbox_plan::KernelParameters;
use toolbox_plan::sdboot::{LoaderEntry, LoaderSettings, SdBootMenu, sections};
use toolbox_state::boot::KernelCmdline;

use super::{rows, view};

/// Files of the Garuda Lima image.
const LOADER_CONF: &str = include_str!("../../../../plan/src/sdboot/fixtures/loader.conf");
const KERNEL_CMDLINE: &str = include_str!("../../../../state/src/boot/fixtures/kernel-cmdline");
const UKI: &[u8] =
    include_bytes!("../../../../plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head");
const FALLBACK: &[u8] =
    include_bytes!("../../../../plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head");

fn os_release(image: &[u8]) -> String {
    let osrel = sections(image)
        .into_iter()
        .find(|section| section.name == ".osrel")
        .unwrap();

    let start = usize::try_from(osrel.offset).unwrap();

    let end = start + usize::try_from(osrel.size).unwrap();

    String::from_utf8(image[start..end].to_vec()).unwrap()
}

fn image_menu(conf: &str) -> SdBootMenu {
    SdBootMenu {
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
    }
}

fn cmdline() -> KernelCmdline {
    KernelCmdline(KernelParameters::parse(KERNEL_CMDLINE).unwrap())
}

#[test]
fn the_images_menu_shows_systemd_boots_defaults_and_its_command_line() {
    let shown = serde_json::to_value(view(&image_menu(LOADER_CONF), Some(&cmdline()))).unwrap();

    assert_eq!(shown["loader"], "systemd-boot");
    assert!(
        shown["timeout"].is_null() && shown["default"].is_null(),
        "{shown}"
    );
    assert_eq!(shown["parameters"][5], "console=tty0");
}

/// The image's `loader.conf` with its commented timeout switched on.
#[test]
fn a_set_timeout_is_shown_as_loader_conf_spells_it() {
    let conf = LOADER_CONF.replace("#timeout", "timeout");

    assert_eq!(view(&image_menu(&conf), None).timeout.as_deref(), Some("3"));
}

#[test]
fn both_ukis_of_the_image_can_be_the_default() {
    let shown: Vec<(String, String)> = rows(&image_menu(LOADER_CONF))
        .into_iter()
        .map(|row| (row.entry, row.title))
        .collect();

    assert_eq!(
        shown,
        [
            (
                "garuda-7.3.1-1-garuda-fallback.efi".to_owned(),
                "Garuda Linux (fallback initramfs)".to_owned()
            ),
            (
                "garuda-7.3.1-1-garuda.efi".to_owned(),
                "Garuda Linux".to_owned()
            ),
        ]
    );
}
