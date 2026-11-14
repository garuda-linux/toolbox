use std::collections::BTreeSet;

use toolbox_core::domains::{CurrentSystemLocale, EnabledLocales};
use toolbox_core::features::locales::{Refusal, SupportedLocales, parse_supported};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{LocaleEntry, LocaleName};

use super::Locales;
use crate::command::{AppCommand, LocaleCommand};
use crate::page::PageId;
use crate::page_service::PageService as _;

fn entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

/// `en_US` is the one enabled locale and the system language.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SupportedLocales(parse_supported(
        "de_DE.UTF-8 UTF-8\nen_US.UTF-8 UTF-8\n",
    )));
    snapshot.insert(EnabledLocales(BTreeSet::from([entry("en_US.UTF-8 UTF-8")])));
    snapshot.insert(CurrentSystemLocale(Some(
        LocaleName::new("en_US.UTF-8").unwrap(),
    )));
    snapshot
}

fn toggle(line: &str, on: bool) -> AppCommand {
    AppCommand::Locale(LocaleCommand::Toggle {
        entry: entry(line),
        on,
    })
}

#[test]
fn turning_off_the_system_language_is_refused_with_its_name() {
    let mut locales = Locales::default();

    let handled = locales.handle(
        &toggle("en_US.UTF-8 UTF-8", false),
        &ChangeSet::default(),
        &system(),
    );

    assert!(handled.edit.is_none());
    let refused = locales.refusal().unwrap();
    assert_eq!(refused.refusal, Refusal::SystemLocale);
    assert_eq!(refused.subject, "en_US.UTF-8");
}

#[test]
fn an_enabled_locale_is_an_edit_and_dismiss_clears_the_refusal() {
    let mut locales = Locales::default();
    locales.handle(
        &toggle("en_US.UTF-8 UTF-8", false),
        &ChangeSet::default(),
        &system(),
    );

    let handled = locales.handle(
        &toggle("de_DE.UTF-8 UTF-8", true),
        &ChangeSet::default(),
        &system(),
    );

    assert!(handled.edit.is_some_and(|edit| edit.len() == 1));
    assert!(locales.refusal().is_none());

    locales.receive(
        &AppCommand::DismissRefusal(PageId::Locales),
        &ChangeSet::default(),
        &system(),
    );
    assert!(locales.refusal().is_none());
}
