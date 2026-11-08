use toolbox_core::Summary;

use super::{sentence, sentence_in};
use crate::i18n;

const CODES: [&str; 58] = [
    "account-lock",
    "account-unlock",
    "aur-install",
    "clean-package-cache",
    "dkms-build",
    "dns",
    "driver-allow",
    "driver-blacklist",
    "full-name-clear",
    "full-name-set",
    "garuda-update-flag-off",
    "garuda-update-flag-on",
    "group-add",
    "group-join",
    "group-leave",
    "group-remove",
    "grub-background",
    "grub-background-none",
    "grub-default",
    "grub-default-saved",
    "grub-os-prober-off",
    "grub-os-prober-on",
    "grub-parameters",
    "grub-parameters-none",
    "grub-save-default-off",
    "grub-save-default-on",
    "grub-submenus-off",
    "grub-submenus-on",
    "grub-theme",
    "grub-theme-none",
    "grub-timeout",
    "hblock-off",
    "hblock-on",
    "home-folder",
    "locale-disable",
    "locale-enable",
    "login-shell",
    "package-install",
    "package-remove",
    "plymouth-theme",
    "refresh-keyring",
    "remove-database-lock",
    "remove-orphans",
    "system-locale",
    "unit-disable",
    "unit-enable",
    "unit-mask",
    "unit-unmask",
    "upgrade-system",
    "user-config-off",
    "user-config-on",
    "user-remove-keep-home",
    "user-remove-with-home",
    "user-unit-disable",
    "user-unit-enable",
    "user-unit-mask",
    "user-unit-unmask",
    "wifi-backend",
];

#[test]
fn every_change_code_has_a_sentence_in_every_language() {
    let ctx = egui::Context::default();

    for language in i18n::available() {
        i18n::set_language(&ctx, &language);
        for code in CODES {
            let summary = Summary::pair(code, "first", "second");
            let id = format!("change-{code}");
            assert_ne!(sentence(&ctx, &summary), id, "{code} in {language}");
        }
    }
}

#[test]
fn a_single_value_fills_the_name() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(&ctx, &Summary::of("package-install", "firefox")),
        "Install firefox"
    );
}

#[test]
fn two_values_fill_their_own_names() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(&ctx, &Summary::pair("group-join", "nico", "wheel")),
        "Add nico to the group wheel"
    );
}

#[test]
fn numbers_pick_the_plural_form() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(&ctx, &Summary::of("grub-timeout", "1")),
        "Show the boot menu for 1 second"
    );
    assert_eq!(
        sentence(&ctx, &Summary::of("grub-timeout", "5")),
        "Show the boot menu for 5 seconds"
    );
}

#[test]
fn a_code_without_a_sentence_shows_its_id() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(&ctx, &Summary::plain("no-such-code")),
        "change-no-such-code"
    );
}

/// The command line has no window, so it names the language itself.
#[test]
fn a_language_named_directly_words_the_same_change() {
    let ctx = egui::Context::default();
    let german: fluent_templates::LanguageIdentifier = "de-DE".parse().unwrap();
    i18n::set_language(&ctx, &german);

    let summary = Summary::pair("group-join", "nico", "wheel");

    assert_eq!(sentence_in(&german, &summary), sentence(&ctx, &summary));
}
