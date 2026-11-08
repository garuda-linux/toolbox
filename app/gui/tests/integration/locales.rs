//! The Languages page with the real queue: a locale switched on, the
//! language packs of the new locale, and the snapshots.

use std::collections::{BTreeMap, BTreeSet};

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::domains::{CurrentSystemLocale, EnabledLocales, InstalledPackages, SyncPackages};
use toolbox_core::features::locales::{
    LanguageName, LocaleNames, SupportedLocales, parse_supported,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::{LocaleEntry, LocaleName};

use crate::common::{Fixture, TALL, packages, window};
use crate::system::first_read;

fn name(language: &str, territory: &str) -> LanguageName {
    LanguageName {
        language: language.to_owned(),
        territory: territory.to_owned(),
    }
}

/// English enabled and the system language, German and French offered,
/// Firefox installed with packs for each.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SupportedLocales(parse_supported(
        "de_DE.UTF-8 UTF-8\nde_DE ISO-8859-1\nen_US.UTF-8 UTF-8\nfr_FR.UTF-8 UTF-8\n",
    )));
    snapshot.insert(EnabledLocales(BTreeSet::from([LocaleEntry::parse(
        "en_US.UTF-8 UTF-8",
    )
    .unwrap()])));
    snapshot.insert(CurrentSystemLocale(Some(
        LocaleName::new("en_US.UTF-8").unwrap(),
    )));
    snapshot.insert(LocaleNames(BTreeMap::from([
        ("de_DE".to_owned(), name("German", "Germany")),
        (
            "en_US".to_owned(),
            name("American English", "United States"),
        ),
        ("fr_FR".to_owned(), name("French", "France")),
    ])));
    snapshot.insert(SyncPackages(packages(&[
        "firefox-i18n-de",
        "firefox-i18n-en-us",
        "firefox-i18n-fr",
    ])));
    snapshot.insert(InstalledPackages(packages(&["firefox"])));
    snapshot
}

pub fn languages() -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let probed = system();
    let mut harness = window(TALL).build_eframe(move |cc| {
        let probe = move || (probed.clone(), Vec::new());
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                ..Setup::default()
            },
        )
    });

    first_read(&mut harness);
    harness
        .get_by_role_and_label(Role::Tab, "Languages")
        .click();
    harness.run();

    Fixture::new(harness, runtime)
}

#[test]
fn a_new_locale_brings_its_language_packs() {
    let mut fixture = languages();
    let harness = fixture.harness();

    harness
        .get_by_role_and_label(Role::CheckBox, "German (Germany)")
        .click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_some());

    harness
        .get_by_role_and_label(Role::RadioButton, "Language packs")
        .click();
    harness.run();

    let pack = |name| harness.query_by_role_and_label(Role::CheckBox, name);
    assert!(pack("firefox-i18n-de").is_some());
    assert!(pack("firefox-i18n-fr").is_none());
    assert!(harness.query_by_label_contains("Missing: 2").is_some());
}

#[test]
fn the_locale_of_the_system_language_stays_on() {
    let mut fixture = languages();
    let harness = fixture.harness();

    harness
        .get_by_role_and_label(Role::CheckBox, "American English (United States)")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_label_contains("is the system language. Choose another")
            .is_some()
    );
    assert!(harness.query_by_label("change queued").is_none());
}
