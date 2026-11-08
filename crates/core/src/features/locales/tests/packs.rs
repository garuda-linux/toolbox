//! Language packs: a package of a program and a language shows for an
//! enabled locale of that language while the program is installed.

use std::collections::BTreeSet;

use super::super::language_packs;
use super::entries;
use crate::domains::{InstalledPackages, SyncPackages};
use crate::testing::packages;

#[test]
fn packs_fit_the_language_and_an_installed_program() {
    let offered = SyncPackages(packages(&[
        "firefox-developer-edition-i18n-de",
        "firefox-i18n-de",
        "firefox-i18n-en-gb",
        "firefox-i18n-en-us",
        "firefox-i18n-pt-br",
        "gimp-help-de",
        "hunspell-de",
        "hunspell-en_us",
        "libreoffice-fresh-de",
        "libreoffice-still-de",
        "man-pages-de",
        "thunderbird-i18n-de",
        "xorg-xmode",
    ]));

    let installed = InstalledPackages(packages(&[
        "firefox",
        "firefox-developer-edition",
        "gimp",
        "hunspell",
        "hunspell-de",
        "libreoffice-fresh",
        "xorg-xmo",
    ]));

    let packs = language_packs(
        &entries(&[
            "de_DE.UTF-8 UTF-8",
            "de_AT.UTF-8 UTF-8",
            "en_US.UTF-8 UTF-8",
            "C.UTF-8 UTF-8",
        ]),
        &offered,
        &installed,
    );

    let shown: Vec<(&str, &str, &str)> = packs
        .iter()
        .map(|pack| {
            (
                pack.package.as_str(),
                pack.base.as_str(),
                pack.locale.as_str(),
            )
        })
        .collect();

    assert_eq!(
        shown,
        [
            (
                "firefox-developer-edition-i18n-de",
                "firefox-developer-edition",
                "de_AT.UTF-8"
            ),
            ("firefox-i18n-de", "firefox", "de_AT.UTF-8"),
            ("firefox-i18n-en-us", "firefox", "en_US.UTF-8"),
            ("gimp-help-de", "gimp", "de_AT.UTF-8"),
            ("hunspell-de", "hunspell", "de_AT.UTF-8"),
            ("hunspell-en_us", "hunspell", "en_US.UTF-8"),
            ("libreoffice-fresh-de", "libreoffice-fresh", "de_AT.UTF-8"),
        ]
    );
}

#[test]
fn no_enabled_locale_or_no_repository_gives_no_packs() {
    let offered = SyncPackages(packages(&["hunspell-de"]));
    let installed = InstalledPackages(packages(&["hunspell"]));

    assert_eq!(language_packs(&BTreeSet::new(), &offered, &installed), []);
    assert_eq!(
        language_packs(
            &entries(&["de_DE.UTF-8 UTF-8"]),
            &SyncPackages::default(),
            &installed
        ),
        []
    );
}
