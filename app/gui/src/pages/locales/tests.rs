mod look;
mod packs;
mod unread;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentSystemLocale, EnabledLocales, InstalledPackages, SyncPackages};
use toolbox_core::features::locales::{
    LanguageName, LocaleNames, Refusal, SupportedLocales, parse_supported,
};
use toolbox_plan::{LocaleEntry, LocaleName, PackageName};

use super::LocalesPage;
use crate::command::{AppCommand, Command, LocaleCommand};
use crate::locales::Locales;
use crate::page::PageId;
use crate::page_service::Services;
use crate::refused::Refused;
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

fn page(snapshot: Snapshot) -> Shown {
    page_with(snapshot, Locales::default())
}

fn page_with(snapshot: Snapshot, locales: Locales) -> Shown {
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        services: Services {
            locales,
            ..Services::default()
        },
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
    let harness = page(Snapshot::default());

    assert!(
        harness
            .query_by_label("Locales are not available on this system")
            .is_some()
    );
}

#[test]
fn a_locale_button_queues_its_locale() {
    let mut harness = page(system());

    harness
        .get_by_role_and_label(Role::Button, "Enable de_DE.UTF-8")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(LocaleCommand::Toggle {
            entry: entry("de_DE.UTF-8 UTF-8"),
            on: toolbox_core::Toggle::On,
        })]
    );
}

#[test]
fn other_character_sets_show_once_utf8_only_is_off() {
    let mut harness = page(system());

    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::Button, "Enable de_DE")
            .count(),
        0
    );

    harness.get_by_label("UTF-8 Only").click();
    harness.run();

    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::Button, "Enable de_DE")
            .count(),
        1
    );
}

#[test]
fn a_search_without_a_match_says_so() {
    let mut harness = page(system());

    harness.get_by_label("Search locales and packs").click();
    harness.run();
    harness
        .get_by_label("Search locales and packs")
        .type_text("klingon");
    harness.run();

    assert!(harness.query_by_label("No locale matches").is_some());
}

#[test]
fn the_system_language_is_picked_among_the_enabled_locales() {
    let mut harness = page(system());

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
    let mut locales = Locales::default();
    locales.set_refusal(Some(Refused {
        refusal: Refusal::SystemLocale,
        subject: "en_US.UTF-8".to_owned(),
    }));

    let mut harness = page_with(system(), locales);

    assert!(
        harness
            .query_by_label_contains("en_US.UTF-8 is the system language")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(AppCommand::DismissRefusal(PageId::Locales))]
    );
}

#[test]
fn the_packs_tab_counts_and_queues_the_missing_packs() {
    let mut harness = page(system());

    harness
        .get_by_role_and_label(Role::RadioButton, "Language Packs")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_label_contains("One pack is not installed.")
            .is_some()
    );

    harness.get_by_label("Queue All Missing").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Install firefox-i18n-en-us")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            Command::from(LocaleCommand::AddMissingPacks),
            Command::from(LocaleCommand::TogglePack {
                package: PackageName::new("firefox-i18n-en-us").unwrap(),
                on: toolbox_core::Toggle::On,
            }),
        ]
    );
}

#[test]
fn the_header_stays_put_and_the_toolbar_holds_search_or_sort() {
    let mut harness = page(system());
    let texts = (
        "Languages",
        "The languages, number and date formats this system can use, and translations for your programs.",
    );

    let tab = rect_of(&harness, Role::RadioButton, "Locales");
    let search = rect_of(&harness, Role::TextInput, "Search locales and packs");

    assert_header(&harness, texts, Some(tab), Some(search));

    harness
        .get_by_role_and_label(Role::RadioButton, "Language Packs")
        .click();
    harness.run();

    assert_eq!(rect_of(&harness, Role::RadioButton, "Locales"), tab);
    let refresh = rect_of(&harness, Role::Button, "Read Again");
    let sort = rect_of(&harness, Role::ComboBox, "Sort language packs");
    let field = rect_of(&harness, Role::TextInput, "Search locales and packs");
    assert!(
        field.right() <= sort.left(),
        "search {field:?} is not before the sort {sort:?}"
    );
    assert!(
        sort.right() <= refresh.left(),
        "sort {sort:?} is not before refresh {refresh:?}"
    );
    assert!(
        (sort.center().y - refresh.center().y).abs() < 0.5,
        "the sort is not on the title row"
    );
}

#[test]
fn a_code_that_names_its_charset_does_not_repeat_it() {
    let harness = page(system());

    assert!(harness.query_by_label_contains("UTF-8, UTF-8").is_none());
}
