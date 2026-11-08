use std::collections::{BTreeMap, BTreeSet};

use super::{
    LanguagePackRow, LocaleRow, add_missing_packs, drop_stranded_packs, locales as all_locales,
    locales_known, pack_rows, packs as pack_list, packs_known, packs_missing as missing,
    system_choice_names, system_choices, system_current, system_queued, system_shown, toggle_pack,
};
use crate::domains::{
    CurrentSystemLocale, EnabledLocales, InstalledPackages, Locales, Packages, SyncPackages,
    SystemLocales,
};
use crate::features::locales::{
    Filter, LanguageName, LocaleNames, PackOrder, SupportedLocales, pack_order, parse_supported,
    pick, toggle,
};
use crate::testing::{locale, locale_entry, package, packages};
use crate::{ChangeSet, Single, Snapshot, Toggle};

fn locales(snapshot: &Snapshot, changes: &ChangeSet, filter: &Filter) -> Vec<LocaleRow> {
    let (rows, shown) = all_locales(snapshot, changes);
    let mut kept: Vec<Option<LocaleRow>> = shown.into_iter().map(Some).collect();

    pick(&rows, filter)
        .into_iter()
        .filter_map(|at| kept.get_mut(at)?.take())
        .collect()
}

fn packs(snapshot: &Snapshot, changes: &ChangeSet, order: PackOrder) -> Vec<LanguagePackRow> {
    let rows = pack_rows(snapshot, changes);
    let mut kept: Vec<Option<LanguagePackRow>> = pack_list(&rows).into_iter().map(Some).collect();

    pack_order(&rows, order)
        .into_iter()
        .filter_map(|at| kept.get_mut(at)?.take())
        .collect()
}

fn packs_missing(snapshot: &Snapshot, changes: &ChangeSet) -> usize {
    missing(&pack_rows(snapshot, changes))
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(SupportedLocales(parse_supported(
        "de_DE.UTF-8 UTF-8\nde_DE ISO-8859-1\nen_US.UTF-8 UTF-8\n",
    )));
    snapshot.insert(EnabledLocales(BTreeSet::from([locale_entry(
        "en_US.UTF-8 UTF-8",
    )])));
    snapshot.insert(CurrentSystemLocale(Some(locale("en_US.UTF-8"))));
    snapshot.insert(LocaleNames(BTreeMap::from([(
        "en_US".to_owned(),
        LanguageName {
            language: "American English".to_owned(),
            territory: "United States".to_owned(),
        },
    )])));
    snapshot.insert(SyncPackages(packages(&[
        "firefox-i18n-de",
        "firefox-i18n-en-us",
        "hunspell-en_us",
    ])));
    snapshot.insert(InstalledPackages(packages(&[
        "firefox",
        "hunspell",
        "hunspell-en_us",
    ])));
    snapshot
}

#[test]
fn a_system_without_generated_locales_is_unknown() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentSystemLocale(Some(locale("en_US.UTF-8"))));
    let changes = ChangeSet::default();

    assert!(!locales_known(&snapshot));
    assert_eq!(
        locales(&snapshot, &changes, &Filter::default()),
        Vec::<LocaleRow>::new()
    );
    assert_eq!(system_current(&snapshot), "en_US.UTF-8");
    assert_eq!(system_queued(&changes), "");
    assert_eq!(system_choices(&snapshot, &changes), Vec::<String>::new());
    assert!(!packs_known(&snapshot, &changes));
    assert_eq!(packs_missing(&snapshot, &changes), 0);
    assert_eq!(
        packs(&snapshot, &changes, PackOrder::Package),
        Vec::<LanguagePackRow>::new()
    );
}

#[test]
fn rows_show_the_state_and_the_queued_wish() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot);

    assert!(locales_known(&snapshot));
    assert_eq!(
        locales(&snapshot, &changes, &Filter::default()),
        [
            LocaleRow {
                locale_code: "en_US.UTF-8".to_owned(),
                charset: "UTF-8".to_owned(),
                language: "American English".to_owned(),
                territory: "United States".to_owned(),
                enabled_now: true,
                on: true,
                queued: "",
                system: true,
            },
            LocaleRow {
                locale_code: "de_DE.UTF-8".to_owned(),
                charset: "UTF-8".to_owned(),
                language: String::new(),
                territory: String::new(),
                enabled_now: false,
                on: true,
                queued: "on",
                system: false,
            },
        ]
    );
}

#[test]
fn the_system_language_offers_enabled_and_queued_locales() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot);
    changes.want::<SystemLocales>(Single, locale("de_DE.UTF-8"), &snapshot);

    assert_eq!(system_current(&snapshot), "en_US.UTF-8");
    assert_eq!(system_queued(&changes), "de_DE.UTF-8");
    assert_eq!(
        system_shown(&snapshot, &changes),
        "de_DE.UTF-8",
        "the queued language wins"
    );
    assert_eq!(
        system_shown(&snapshot, &ChangeSet::default()),
        "en_US.UTF-8",
        "without a queued language the current one shows"
    );
    assert_eq!(
        system_choices(&snapshot, &changes),
        ["de_DE.UTF-8", "en_US.UTF-8"]
    );
}

#[test]
fn packs_follow_the_enabled_locales_and_count_what_is_missing() {
    let snapshot = snapshot();
    let changes = ChangeSet::default();

    assert!(packs_known(&snapshot, &changes));
    assert_eq!(packs_missing(&snapshot, &changes), 1);
    assert_eq!(
        packs(&snapshot, &changes, PackOrder::Package),
        [
            LanguagePackRow {
                pack: "firefox-i18n-en-us".to_owned(),
                base: "firefox".to_owned(),
                locale_code: "en_US.UTF-8".to_owned(),
                language: "American English".to_owned(),
                territory: "United States".to_owned(),
                installed: false,
                on: false,
                queued: "",
            },
            LanguagePackRow {
                pack: "hunspell-en_us".to_owned(),
                base: "hunspell".to_owned(),
                locale_code: "en_US.UTF-8".to_owned(),
                language: "American English".to_owned(),
                territory: "United States".to_owned(),
                installed: true,
                on: true,
                queued: "",
            },
        ]
    );

    let mut queued = ChangeSet::default();
    queued.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot);
    queued.want::<Packages>(package("firefox-i18n-en-us"), Toggle::On, &snapshot);

    let listed = packs(&snapshot, &queued, PackOrder::Package);

    assert_eq!(packs_missing(&snapshot, &queued), 1);
    assert_eq!(listed[0].pack, "firefox-i18n-de");
    assert_eq!(listed[1].queued, "on");
    assert!(listed[1].on);
}

#[test]
fn the_pack_order_is_followed() {
    let snapshot = snapshot();
    let changes = ChangeSet::default();
    let listed = packs(&snapshot, &changes, PackOrder::Selected);

    assert_eq!(listed[0].pack, "hunspell-en_us");
    assert_eq!(listed[1].pack, "firefox-i18n-en-us");
}

#[test]
fn the_search_finds_a_locale_by_its_language_name() {
    let filter = Filter {
        search: "american".to_owned(),
        ..Filter::default()
    };

    let listed = locales(&snapshot(), &ChangeSet::default(), &filter);

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].locale_code, "en_US.UTF-8");
}

#[test]
fn every_system_choice_carries_its_language_where_one_is_known() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot);

    assert_eq!(
        system_choice_names(&snapshot, &changes),
        [
            (String::new(), String::new()),
            ("American English".to_owned(), "United States".to_owned()),
        ]
    );
}

#[test]
fn a_pack_queued_for_removal_is_not_missing_and_queue_all_takes_only_missing_ones() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("hunspell-en_us"), Toggle::Off, &snapshot);

    assert_eq!(packs_missing(&snapshot, &changes), 1);
    assert_eq!(add_missing_packs(&mut changes, &snapshot), 1);
    assert_eq!(packs_missing(&snapshot, &changes), 0);
    assert_eq!(
        changes.wish::<Packages>(&package("hunspell-en_us")),
        Some(&Toggle::Off)
    );
    assert_eq!(add_missing_packs(&mut changes, &snapshot), 0);
}

#[test]
fn only_listed_packs_are_queued() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert!(!toggle_pack(&package("vim"), true, &mut changes, &snapshot));
    assert!(toggle_pack(
        &package("firefox-i18n-en-us"),
        true,
        &mut changes,
        &snapshot
    ));
    assert_eq!(
        changes.wish::<Packages>(&package("firefox-i18n-en-us")),
        Some(&Toggle::On)
    );
}

#[test]
fn a_locale_taken_back_takes_its_queued_packs_with_it() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let german = locale_entry("de_DE.UTF-8 UTF-8");
    toggle(&german, true, &mut changes, &snapshot).unwrap();
    assert!(toggle_pack(
        &package("firefox-i18n-de"),
        true,
        &mut changes,
        &snapshot
    ));
    assert!(toggle_pack(
        &package("firefox-i18n-en-us"),
        true,
        &mut changes,
        &snapshot
    ));

    toggle(&german, false, &mut changes, &snapshot).unwrap();

    assert_eq!(changes.wish::<Packages>(&package("firefox-i18n-de")), None);
    assert_eq!(
        changes.wish::<Packages>(&package("firefox-i18n-en-us")),
        Some(&Toggle::On),
        "the pack of a locale that stays keeps its install"
    );
}

#[test]
fn a_restored_queue_loses_the_packs_of_locales_it_no_longer_has() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("firefox-i18n-de"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("firefox-i18n-en-us"), Toggle::On, &snapshot);

    assert_eq!(drop_stranded_packs(&mut changes, &snapshot), Some(1));
    assert_eq!(changes.wish::<Packages>(&package("firefox-i18n-de")), None);
    assert_eq!(
        drop_stranded_packs(&mut changes, &Snapshot::default()),
        None,
        "without the package lists nothing is checked"
    );
}
