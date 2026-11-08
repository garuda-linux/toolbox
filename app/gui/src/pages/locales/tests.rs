mod look;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentSystemLocale, EnabledLocales, InstalledPackages, SyncPackages};
use toolbox_core::features::locales::{
    LanguageName, LocaleNames, SupportedLocales, parse_supported,
};
use toolbox_plan::{LocaleEntry, LocaleName, PackageName};

use super::LocalesPage;
use crate::command::{Command, LocaleCommand};
use crate::pages::{LocaleRefusal, Refusals};
use crate::testing::{PageFixture, PageState, assert_header, rect_of};

type Shown = Harness<'static, PageState<LocalesPage>>;

fn entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

fn packages(names: &[&str]) -> BTreeSet<PackageName> {
    names
        .iter()
        .map(|name| PackageName::new(*name).unwrap())
        .collect()
}

fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SupportedLocales(parse_supported(
        "de_DE.UTF-8 UTF-8\nde_DE ISO-8859-1\nen_US.UTF-8 UTF-8\n",
    )));
    snapshot.insert(EnabledLocales(BTreeSet::from([entry("en_US.UTF-8 UTF-8")])));
    snapshot.insert(CurrentSystemLocale(Some(
        LocaleName::new("en_US.UTF-8").unwrap(),
    )));
    snapshot.insert(LocaleNames(BTreeMap::from([(
        "en_US".to_owned(),
        LanguageName {
            language: "American English".to_owned(),
            territory: "United States".to_owned(),
        },
    )])));
    snapshot.insert(SyncPackages(packages(&["firefox-i18n-en-us"])));
    snapshot.insert(InstalledPackages(packages(&["firefox"])));
    snapshot
}

fn page(snapshot: Snapshot, refusals: Refusals) -> Shown {
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        refusals,
        ..PageFixture::default()
    }
    .harness(LocalesPage::default());

    harness.run();
    harness
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn a_system_without_locale_gen_says_so() {
    let harness = page(Snapshot::default(), Refusals::default());

    assert!(
        harness
            .query_by_label("Locales are not available on this system")
            .is_some()
    );
}

#[test]
fn a_switch_queues_its_locale() {
    let mut harness = page(system(), Refusals::default());

    harness
        .get_by_role_and_label(Role::CheckBox, "de_DE.UTF-8")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(LocaleCommand::Toggle {
            entry: entry("de_DE.UTF-8 UTF-8"),
            on: true,
        })]
    );
}

#[test]
fn other_character_sets_show_once_utf8_only_is_off() {
    let mut harness = page(system(), Refusals::default());

    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::CheckBox, "de_DE")
            .count(),
        0
    );

    harness.get_by_label("UTF-8 only").click();
    harness.run();

    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::CheckBox, "de_DE")
            .count(),
        1
    );
}

#[test]
fn a_search_without_a_match_says_so() {
    let mut harness = page(system(), Refusals::default());

    harness.get_by_label("Search locales").click();
    harness.run();
    harness.get_by_label("Search locales").type_text("klingon");
    harness.run();

    assert!(harness.query_by_label("No locale matches").is_some());
}

#[test]
fn the_system_language_is_picked_among_the_enabled_locales() {
    let mut harness = page(system(), Refusals::default());

    harness
        .get_by_role_and_label(Role::ComboBox, "System language")
        .click();
    harness.run();
    harness
        .get_by_label("American English (United States), en_US.UTF-8")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(LocaleCommand::System(
            LocaleName::new("en_US.UTF-8").unwrap()
        ))]
    );
}

#[test]
fn a_refused_click_names_the_locale_until_dismissed() {
    let refusals = Refusals {
        locale: Some(LocaleRefusal {
            code: "system-locale",
            locale: "en_US.UTF-8".to_owned(),
        }),
        ..Refusals::default()
    };

    let mut harness = page(system(), refusals);

    assert!(
        harness
            .query_by_label_contains("en_US.UTF-8 is the system language")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(LocaleCommand::DismissRefusal)]
    );
}

#[test]
fn the_packs_tab_counts_and_queues_the_missing_packs() {
    let mut harness = page(system(), Refusals::default());

    harness
        .get_by_role_and_label(Role::RadioButton, "Language packs")
        .click();
    harness.run();

    assert!(harness.query_by_label_contains("Missing: 1").is_some());

    harness.get_by_label("Queue all missing").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "firefox-i18n-en-us")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            Command::from(LocaleCommand::AddMissingPacks),
            Command::from(LocaleCommand::TogglePack {
                package: PackageName::new("firefox-i18n-en-us").unwrap(),
                on: true,
            }),
        ]
    );
}

#[test]
fn the_header_stays_put_and_the_toolbar_holds_search_or_sort() {
    let mut harness = page(system(), Refusals::default());
    let texts = (
        "Languages",
        "The languages, number and date formats this system can use, and translations for your programs.",
    );

    let tab = rect_of(&harness, Role::RadioButton, "Locales");
    let search = rect_of(&harness, Role::TextInput, "Search locales");
    assert_header(&harness, texts, Some(tab), Some(search));

    harness
        .get_by_role_and_label(Role::RadioButton, "Language packs")
        .click();
    harness.run();

    assert_eq!(rect_of(&harness, Role::RadioButton, "Locales"), tab);
    let refresh = rect_of(&harness, Role::Button, "Read again");

    let sort = rect_of(&harness, Role::ComboBox, "Sort language packs");
    assert!(
        (sort.right() - refresh.right()).abs() < 0.5,
        "sort {sort:?}, refresh {refresh:?}"
    );
    assert!(
        sort.top() > refresh.bottom(),
        "the sort is not in the toolbar"
    );
}
