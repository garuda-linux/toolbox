use std::collections::BTreeSet;

use toolbox_plan::{PlannedAction, RootAction};

use super::{CurrentSystemLocale, EnabledLocales, Locales, SystemLocales};
use crate::domain::{Summary, Toggle};
use crate::domains::Packages;
use crate::testing::{actions, locale, locale_entry, package, restored, views};
use crate::{ChangeSet, Single, Snapshot};

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(EnabledLocales(BTreeSet::from([locale_entry(
        "en_US.UTF-8 UTF-8",
    )])));
    snapshot.insert(CurrentSystemLocale(Some(locale("en_US.UTF-8"))));
    snapshot
}

#[test]
fn a_locale_as_it_is_drops_out() {
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("en_US.UTF-8 UTF-8"), Toggle::On, &snapshot());
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::Off, &snapshot());
    changes.want::<SystemLocales>(Single, locale("en_US.utf8"), &snapshot());

    assert!(changes.is_empty());
}

#[test]
fn all_edits_come_first_then_one_locale_gen_then_the_system_locale() {
    let mut changes = ChangeSet::default();
    changes.want::<SystemLocales>(Single, locale("de_DE.UTF-8"), &snapshot());
    changes.want::<Locales>(locale_entry("en_US.UTF-8 UTF-8"), Toggle::Off, &snapshot());
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot());
    changes.want::<Packages>(package("hunspell-de"), Toggle::On, &snapshot());

    assert_eq!(
        actions(&changes)[1..],
        [
            PlannedAction::Root(RootAction::SetLocaleEnabled {
                locale: locale_entry("de_DE.UTF-8 UTF-8"),
                enabled: true,
            }),
            PlannedAction::Root(RootAction::SetLocaleEnabled {
                locale: locale_entry("en_US.UTF-8 UTF-8"),
                enabled: false,
            }),
            PlannedAction::Root(RootAction::GenerateLocales),
            PlannedAction::Root(RootAction::SetSystemLocale {
                locale: locale("de_DE.UTF-8"),
            }),
        ]
    );
    assert!(matches!(
        actions(&changes)[0],
        PlannedAction::Root(RootAction::InstallPackages { packages: _ })
    ));
}

#[test]
fn the_review_page_gets_the_line_and_the_new_language() {
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot());
    changes.want::<SystemLocales>(Single, locale("de_DE.UTF-8"), &snapshot());

    assert_eq!(
        views(&changes),
        [
            (
                "locales",
                "de_DE.UTF-8 UTF-8".to_owned(),
                Summary::of("locale-enable", "de_DE.UTF-8 UTF-8")
            ),
            (
                "system-locale",
                "system-locale".to_owned(),
                Summary::of("system-locale", "de_DE.UTF-8")
            ),
        ]
    );
}

#[test]
fn an_applied_change_leaves_the_queue_with_the_next_snapshot() {
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("de_DE.UTF-8 UTF-8"), Toggle::On, &snapshot());
    changes.want::<SystemLocales>(Single, locale("de_DE.UTF-8"), &snapshot());

    let mut after = Snapshot::default();
    after.insert(EnabledLocales(BTreeSet::from([
        locale_entry("en_US.UTF-8 UTF-8"),
        locale_entry("de_DE.UTF-8 UTF-8"),
    ])));
    after.insert(CurrentSystemLocale(Some(locale("de_DE.UTF-8"))));
    changes.reconcile(&after);

    assert!(changes.is_empty());
}

#[test]
fn locale_changes_survive_a_save_and_restore() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<Locales>(locale_entry("sr_RS@latin UTF-8"), Toggle::On, &empty);
    changes.want::<SystemLocales>(Single, locale("sr_RS@latin"), &empty);

    let (restored, problems) = restored(&changes, &empty);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
}
