use super::{LoaderSettings, LoaderTimeout};
use crate::LoaderDefault;

/// `/efi/loader/loader.conf` of the Garuda Lima image, as `bootctl
/// install` writes it.
const IMAGE: &str = crate::testing::sdboot::LOADER_CONF;

/// What garuda-common-settings' config agent rule appends.
const COMMON_SETTINGS: &str = "auto-firmware yes\nauto-reboot yes\nauto-poweroff yes\n";

/// The example of `loader.conf(5)`.
const MAN_PAGE: &str = "timeout 0\ndefault 01234567890abcdef1234567890abdf0-*\neditor no\n";

#[test]
fn the_images_file_leaves_every_default() {
    assert_eq!(LoaderSettings::parse(IMAGE), LoaderSettings::default());
}

#[test]
fn garuda_common_settings_switches_the_firmware_entry_on() {
    let settings = LoaderSettings::parse(&format!("{IMAGE}{COMMON_SETTINGS}"));

    assert_eq!(
        settings,
        LoaderSettings {
            auto_firmware: Some(true),
            ..LoaderSettings::default()
        }
    );
}

#[test]
fn the_man_pages_example_is_read_whole() {
    let settings = LoaderSettings::parse(MAN_PAGE);

    assert_eq!(settings.timeout, Some(LoaderTimeout::Seconds(0)));
    assert_eq!(
        settings.default,
        Some(LoaderDefault::new("01234567890abcdef1234567890abdf0-*").unwrap())
    );
    assert_eq!(settings.editor, Some(false));
}

/// The image's own commented lines, switched on.
#[test]
fn the_images_commented_timeout_counts_once_uncommented() {
    let settings = LoaderSettings::parse(&IMAGE.replace("#timeout", "timeout"));

    assert_eq!(settings.timeout, Some(LoaderTimeout::Seconds(3)));
}

/// The words `loader.conf(5)` lists beside seconds.
#[test]
fn the_timeout_also_takes_the_menu_words() {
    for (value, timeout) in [
        ("menu-force", LoaderTimeout::MenuForce),
        ("menu-hidden", LoaderTimeout::MenuHidden),
        ("menu-disabled", LoaderTimeout::MenuDisabled),
    ] {
        let settings = LoaderSettings::parse(&format!("{IMAGE}timeout {value}\n"));

        assert_eq!(settings.timeout, Some(timeout), "{value}");
    }
}

/// systemd-boot takes the last line of a setting, and ignores a value it
/// cannot read.
#[test]
fn the_last_line_wins_and_an_odd_value_counts_as_unset() {
    let settings = LoaderSettings::parse(&format!("{MAN_PAGE}timeout 9\neditor maybe\n"));

    assert_eq!(settings.timeout, Some(LoaderTimeout::Seconds(9)));
    assert_eq!(settings.editor, None);
}
