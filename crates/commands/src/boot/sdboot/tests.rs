use toolbox_cli::args::BootKey;
use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::{
    LoaderEntry, LoaderSetting, LoaderSettings, LoaderTimeout, SdBootMenu, sections,
};
use toolbox_state::boot::{KernelCmdline, SdBootKernel};

use super::{is_kernel_key, loader_setting, rows, view};
use crate::change::Refused;

/// Files of the Garuda Lima image.
const LOADER_CONF: &str = include_str!("../../../../plan/src/sdboot/fixtures/loader.conf");
const KERNEL_CMDLINE: &str = include_str!("../../../../state/src/boot/fixtures/kernel-cmdline");
/// `/proc/cmdline` of the Garuda live ISO.
const LIVE_CMDLINE: &str = include_str!("../../../../system/src/host/fixtures/live/proc-cmdline");
const UKI: &[u8] =
    include_bytes!("../../../../plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head");
const FALLBACK: &[u8] =
    include_bytes!("../../../../plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head");

fn section(image: &[u8], name: &str) -> String {
    let found = sections(image)
        .into_iter()
        .find(|section| section.name == name)
        .unwrap();

    let start = usize::try_from(found.offset).unwrap();
    let end = start + usize::try_from(found.size).unwrap();

    String::from_utf8(image[start..end].to_vec()).unwrap()
}

fn uki(file: &str, image: &[u8]) -> LoaderEntry {
    LoaderEntry::uki(
        file,
        &section(image, ".osrel"),
        Some(&section(image, ".uname")),
    )
    .unwrap()
}

fn image_menu(conf: &str) -> SdBootMenu {
    SdBootMenu {
        settings: LoaderSettings::parse(conf),
        entries: vec![
            uki("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK),
            uki("garuda-7.3.1-1-garuda.efi", UKI),
        ],
    }
}

fn kernel(line: &str) -> SdBootKernel {
    SdBootKernel {
        cmdline: KernelCmdline::read(line, false),
        theme: None,
    }
}

#[test]
fn the_images_menu_shows_systemd_boots_defaults_and_its_command_line() {
    let shown = serde_json::to_value(view(
        &image_menu(LOADER_CONF),
        Some(&kernel(KERNEL_CMDLINE)),
    ))
    .unwrap();

    assert_eq!(shown["loader"], "systemd-boot");
    assert!(
        shown["timeout"].is_null() && shown["default"].is_null(),
        "{shown}"
    );
    assert_eq!(shown["parameters"][5], "console=tty0");
    assert!(shown["plymouth"].is_null(), "{shown}");
}

#[test]
fn the_live_command_line_shows_word_by_word() {
    let shown = view(&image_menu(LOADER_CONF), Some(&kernel(LIVE_CMDLINE)));

    assert_eq!(
        shown.parameters.unwrap()[0],
        "initrd=\\garuda\\amd-ucode.img"
    );
}

#[test]
fn the_switches_and_plymouth_are_no_loader_conf_settings() {
    for key in [
        BootKey::Quiet,
        BootKey::Splash,
        BootKey::MitigationsOff,
        BootKey::Sysrq,
        BootKey::Plymouth,
    ] {
        assert!(is_kernel_key(key), "{key:?}");
        loader_setting(key, "on", &image_menu(LOADER_CONF)).unwrap_err();
    }

    assert!(!is_kernel_key(BootKey::Editor));
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

/// The image's kernel picked by its id: the default follows the kernel
/// through updates instead of naming this one version.
#[test]
fn a_picked_kernel_becomes_a_default_that_survives_updates() {
    let setting = loader_setting(
        BootKey::Default,
        "garuda-7.3.1-1-garuda.efi",
        &image_menu(LOADER_CONF),
    )
    .unwrap();

    assert_eq!(
        setting,
        LoaderSetting::Default(Some(LoaderDefault::new("garuda-*-garuda.efi").unwrap()))
    );
}

#[test]
fn the_words_of_the_menu_settings_are_taken() {
    let menu = image_menu(LOADER_CONF);
    let set = |key, value| loader_setting(key, value, &menu).unwrap();

    assert_eq!(
        set(BootKey::Timeout, "menu-force"),
        LoaderSetting::Timeout(Some(LoaderTimeout::MenuForce))
    );
    assert_eq!(set(BootKey::Timeout, "none"), LoaderSetting::Timeout(None));
    assert_eq!(
        set(BootKey::Remember, "on"),
        LoaderSetting::Default(Some(LoaderDefault::new("@saved").unwrap()))
    );
    assert_eq!(
        set(BootKey::Editor, "off"),
        LoaderSetting::Editor(Some(false))
    );
    assert_eq!(
        set(BootKey::AutoFirmware, "none"),
        LoaderSetting::AutoFirmware(None)
    );
}

#[test]
fn an_entry_the_menu_lacks_and_grubs_own_settings_are_refused() {
    let menu = image_menu(LOADER_CONF);

    for (key, value) in [
        (BootKey::Default, "garuda-6.1.0-1-lts.efi"),
        (BootKey::Submenus, "on"),
        (BootKey::Timeout, "301"),
    ] {
        assert!(
            matches!(loader_setting(key, value, &menu), Err(Refused::Rule(_))),
            "{key:?} {value}"
        );
    }
}
