//! Language packs: a package of a program and a language shows for an
//! enabled locale of that language while the program is installed.

use std::collections::BTreeSet;

use super::super::{LanguageName, LanguagePack, PackOrder, PackRow, language_packs, pack_pick};
use super::entries;
use crate::domains::{InstalledPackages, SyncPackages};
use crate::test_support::packages;

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

fn pack_row(package: &str, base: &str, code: &str, language: &str, territory: &str) -> PackRow {
    PackRow {
        pack: LanguagePack {
            package: crate::test_support::package(package),
            base: crate::test_support::package(base),
            locale: crate::test_support::locale(code),
        },
        name: Some(LanguageName {
            language: language.to_owned(),
            territory: territory.to_owned(),
        }),
        installed: false,
        queued: None,
    }
}

#[test]
fn the_pack_search_finds_package_program_language_and_country() {
    let rows = [
        pack_row(
            "firefox-i18n-de",
            "firefox",
            "de_DE.UTF-8",
            "German",
            "Germany",
        ),
        pack_row(
            "hunspell-en_us",
            "hunspell",
            "en_US.UTF-8",
            "American English",
            "United States",
        ),
    ];

    let found = |search| pack_pick(&rows, PackOrder::Package, search);

    assert_eq!(found(""), [0, 1]);
    assert_eq!(found("HUNSPELL"), [1]);
    assert_eq!(found("firefox"), [0]);
    assert_eq!(found("german"), [0]);
    assert_eq!(found("united"), [1]);
    assert_eq!(found("de_de"), [0]);
    assert_eq!(found("klingon"), [0_usize; 0]);
}
