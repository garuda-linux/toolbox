use std::collections::BTreeMap;

use super::super::{
    Filter, LanguageName, LanguagePack, LocaleNames, PackOrder, PackRow, pack_order,
    parse_identification, source_name,
};
use super::{all, rows, snapshot};
use crate::test_support::{locale, package};
use crate::{ChangeSet, Snapshot, Toggle};

/// `/usr/share/i18n/locales/de_DE` and `eo` of the Lima VM.
const GERMAN: &str = toolbox_system::testing::locales::LOCALES_DE_DE;
const ESPERANTO: &str = toolbox_system::testing::locales::LOCALES_EO;

fn name(language: &str, territory: &str) -> LanguageName {
    LanguageName {
        language: language.to_owned(),
        territory: territory.to_owned(),
    }
}

fn named() -> Snapshot {
    let mut snapshot = snapshot(&["en_US.UTF-8 UTF-8"], Some("en_US.UTF-8"));

    snapshot.insert(LocaleNames(BTreeMap::from([
        ("de_DE".to_owned(), name("German", "Germany")),
        ("de_DE@euro".to_owned(), name("German", "Germany")),
        ("en_GB".to_owned(), name("English", "United Kingdom")),
        (
            "en_US".to_owned(),
            name("American English", "United States"),
        ),
    ])));
    snapshot
}

fn found(search: &str) -> Vec<String> {
    let filter = Filter {
        search: search.to_owned(),
        ..all()
    };

    rows(&named(), &ChangeSet::default(), &filter)
        .into_iter()
        .map(|row| row.entry.locale.to_string())
        .collect()
}

#[test]
fn language_and_territory_come_from_the_identification_part() {
    assert_eq!(
        parse_identification(GERMAN),
        Some(name("German", "Germany"))
    );
    assert_eq!(parse_identification(ESPERANTO), Some(name("Esperanto", "")));

    // `de_DE` edited to spell a letter as `<U....>`, as older glibc files do.
    let escaped = GERMAN.replace("\"Germany\"", "\"Germ<U0061>ny\"");

    assert_eq!(
        parse_identification(&escaped),
        Some(name("German", "Germany"))
    );
    assert_eq!(
        parse_identification("language \"Norwegian Bokmål\"\nterritory \"Åland <Uzz> <U41\""),
        Some(name("Norwegian Bokmål", "Åland <Uzz> <U41"))
    );
}

#[test]
fn files_that_name_no_language_have_no_name() {
    // `de_DE` edited to name neither language nor territory.
    let unnamed = GERMAN
        .replace("\"German\"", "\"\"")
        .replace("\"Germany\"", "\"\"");

    assert_eq!(parse_identification(&unnamed), None);

    for text in [
        "",
        "LC_IDENTIFICATION\ntitle \"ISO 14651\"\nEND LC_IDENTIFICATION\nlanguage \"late\"\n",
        "languages \"German\"\n",
        "language German\n",
    ] {
        assert_eq!(parse_identification(text), None, "{text:?}");
    }
}

#[test]
fn names_are_plain_and_short() {
    let long = format!("language \"{}\"\n", "x".repeat(500));

    assert_eq!(
        parse_identification(&long).map(|name| name.language.chars().count()),
        Some(64)
    );
    assert_eq!(
        parse_identification("language \" Ger<U001B>[0mman \"\n"),
        Some(name("Ger[0mman", ""))
    );
}

#[test]
fn a_locale_is_named_by_the_source_file_of_its_base_and_modifier() {
    for (locale_name, source) in [
        ("de_DE.UTF-8", "de_DE"),
        ("de_DE", "de_DE"),
        ("de_DE@euro", "de_DE@euro"),
        ("ca_ES.UTF-8@valencia", "ca_ES@valencia"),
        ("sr_RS@latin", "sr_RS@latin"),
        ("eo", "eo"),
    ] {
        assert_eq!(source_name(locale_name), source);
    }

    let system = named();
    let names = system.part::<LocaleNames>().unwrap();

    assert_eq!(
        names.of(&locale("de_DE.UTF-8")),
        Some(&name("German", "Germany"))
    );
    assert_eq!(names.of(&locale("sr_RS@latin")), None);
}

#[test]
fn the_search_finds_a_locale_by_its_language_and_its_territory() {
    assert_eq!(found("german"), ["de_DE.UTF-8", "de_DE", "de_DE@euro"]);
    assert_eq!(found("GERMANY"), ["de_DE.UTF-8", "de_DE", "de_DE@euro"]);
    assert_eq!(found("kingdom"), ["en_GB.UTF-8", "en_GB"]);
    assert_eq!(
        found("english"),
        ["en_US.UTF-8", "en_GB.UTF-8", "en_GB", "en_US"]
    );
    assert_eq!(found("de_de@"), ["de_DE@euro"]);
    assert_eq!(found("serbian"), Vec::<String>::new());
}

#[test]
fn rows_carry_the_name_and_lack_it_where_glibc_has_none() {
    let rows = rows(&named(), &ChangeSet::default(), &all());
    let of = |locale_name: &str| {
        rows.iter()
            .find(|row| row.entry.locale.as_str() == locale_name)
            .unwrap()
            .name
            .clone()
    };

    assert_eq!(of("en_GB.UTF-8"), Some(name("English", "United Kingdom")));
    assert_eq!(of("sr_RS@latin"), None);

    // Without the part every row still shows, by its code.
    let plain = super::rows(
        &snapshot(&["en_US.UTF-8 UTF-8"], None),
        &ChangeSet::default(),
        &all(),
    );

    assert!(plain.iter().all(|row| row.name.is_none()));
    assert_eq!(plain.len(), rows.len());
}

fn pack(package_name: &str, base: &str, locale_name: &str, language: Option<&str>) -> PackRow {
    PackRow {
        pack: LanguagePack {
            package: package(package_name),
            base: package(base),
            locale: locale(locale_name),
        },
        name: language.map(|language| name(language, "")),
        installed: false,
        queued: None,
    }
}

fn ordered(order: PackOrder) -> Vec<String> {
    let mut rows = vec![
        pack("firefox-i18n-de", "firefox", "de_DE.UTF-8", Some("German")),
        pack(
            "hunspell-en_us",
            "hunspell",
            "en_US.UTF-8",
            Some("American English"),
        ),
        pack("thunderbird-i18n-sr", "thunderbird", "sr_RS@latin", None),
        pack("aspell-de", "aspell", "de_DE.UTF-8", Some("German")),
    ];

    if let Some(row) = rows.get_mut(1) {
        row.installed = true;
    }

    if let Some(row) = rows.get_mut(2) {
        row.queued = Some(Toggle::On);
    }

    if let Some(row) = rows.get_mut(3) {
        row.installed = true;
        row.queued = Some(Toggle::Off);
    }

    pack_order(&rows, order)
        .into_iter()
        .map(|at| rows[at].pack.package.to_string())
        .collect()
}

#[test]
fn packs_sort_by_package_program_language_or_selection() {
    assert_eq!(
        ordered(PackOrder::Package),
        [
            "aspell-de",
            "firefox-i18n-de",
            "hunspell-en_us",
            "thunderbird-i18n-sr"
        ]
    );
    assert_eq!(
        ordered(PackOrder::Program),
        [
            "aspell-de",
            "firefox-i18n-de",
            "hunspell-en_us",
            "thunderbird-i18n-sr"
        ]
    );
    assert_eq!(
        ordered(PackOrder::Language),
        [
            "hunspell-en_us",
            "aspell-de",
            "firefox-i18n-de",
            "thunderbird-i18n-sr"
        ]
    );
    assert_eq!(
        ordered(PackOrder::Selected),
        [
            "hunspell-en_us",
            "thunderbird-i18n-sr",
            "aspell-de",
            "firefox-i18n-de"
        ]
    );
}
