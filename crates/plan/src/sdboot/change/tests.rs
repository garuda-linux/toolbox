use super::{InvalidLoaderChange, LoaderChange, LoaderSetting};
use crate::LoaderDefault;
use crate::sdboot::{LoaderEntry, LoaderSettings, LoaderTimeout, sections};

/// `/efi/loader/loader.conf` of the Garuda Lima image, and the lines
/// garuda-common-settings' config agent rule appends to it.
const IMAGE: &str = crate::testing::sdboot::LOADER_CONF;
const COMMON_SETTINGS: &str = "auto-firmware yes\nauto-reboot yes\nauto-poweroff yes\n";

const UKI: &[u8] = crate::testing::sdboot::GARUDA_7_3_1_1_GARUDA_EFI_HEAD;
const FALLBACK: &[u8] = crate::testing::sdboot::GARUDA_7_3_1_1_GARUDA_FALLBACK_EFI_HEAD;

fn change(settings: Vec<LoaderSetting>) -> LoaderChange {
    LoaderChange::new(settings).unwrap()
}

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

#[test]
fn the_images_commented_timeout_is_switched_on_in_its_place() {
    let wanted = change(vec![LoaderSetting::Timeout(Some(LoaderTimeout::Seconds(
        5,
    )))]);

    assert_eq!(wanted.apply(IMAGE), "timeout 5\n#console-mode keep\n");
}

#[test]
fn a_setting_the_file_never_names_is_added_at_the_end() {
    let wanted = change(vec![LoaderSetting::Editor(Some(false))]);

    assert_eq!(wanted.apply(IMAGE), format!("{IMAGE}editor no\n"));
}

/// The image with garuda-common-settings' lines: only the changed one
/// differs, the others stay as the config agent wrote them.
#[test]
fn every_other_line_stays_byte_for_byte() {
    let file = format!("{IMAGE}{COMMON_SETTINGS}");
    let wanted = change(vec![LoaderSetting::AutoFirmware(Some(false))]);

    assert_eq!(
        wanted.apply(&file),
        format!("{IMAGE}auto-firmware no\nauto-reboot yes\nauto-poweroff yes\n")
    );
}

/// Back to systemd-boot's own default: the line stays as a comment, as
/// `/etc/default/grub` keeps an unset variable.
#[test]
fn an_unset_setting_is_commented_out() {
    let file = format!("{IMAGE}{COMMON_SETTINGS}");
    let wanted = change(vec![LoaderSetting::AutoFirmware(None)]);

    assert_eq!(
        wanted.apply(&file),
        format!("{IMAGE}#auto-firmware yes\nauto-reboot yes\nauto-poweroff yes\n")
    );
}

#[test]
fn the_written_file_reads_back_as_the_wanted_settings() {
    let wanted = change(vec![
        LoaderSetting::Timeout(Some(LoaderTimeout::MenuForce)),
        LoaderSetting::Default(Some(LoaderDefault::new("@saved").unwrap())),
        LoaderSetting::AutoEntries(Some(true)),
    ]);

    let written = LoaderSettings::parse(&wanted.apply(IMAGE));

    assert!(wanted.iter().all(|setting| setting.is_met_by(&written)));
    assert_eq!(written.timeout, Some(LoaderTimeout::MenuForce));
}

#[test]
fn a_change_names_each_setting_once() {
    assert_eq!(
        LoaderChange::new(Vec::new()).unwrap_err(),
        InvalidLoaderChange::Empty
    );
    assert_eq!(
        LoaderChange::new(vec![
            LoaderSetting::Editor(Some(true)),
            LoaderSetting::Editor(None),
        ])
        .unwrap_err(),
        InvalidLoaderChange::Repeated("editor")
    );
}

/// The image's UKIs: a default that names the kernel's flavour, not its
/// version, still finds the kernel after the next update.
#[test]
fn a_ukis_default_follows_its_kernel_through_updates() {
    let normal = uki("garuda-7.3.1-1-garuda.efi", UKI);
    let fallback = uki("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK);

    assert_eq!(normal.default_pattern().as_str(), "garuda-*-garuda.efi");
    assert_eq!(
        fallback.default_pattern().as_str(),
        "garuda-*-garuda-fallback.efi"
    );
}

/// The image's UKI without its `.uname`: without the kernel release the
/// version cannot be told apart, so the default names the entry itself.
#[test]
fn an_entry_without_a_kernel_release_is_named_exactly() {
    let bare =
        LoaderEntry::uki("garuda-7.3.1-1-garuda.efi", &section(UKI, ".osrel"), None).unwrap();

    assert_eq!(bare.default_pattern().as_str(), "garuda-7.3.1-1-garuda.efi");
}
