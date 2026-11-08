//! A queue restored from disk never met the page's guards: the locale
//! guards run over it once the enabled locales are known.

use super::{snapshot, toggle};
use crate::domains::Locales;
use crate::features::locales::recheck;
use crate::testing::locale_entry;
use crate::{ChangeSet, Toggle};

#[test]
fn a_restored_switch_off_of_the_system_language_locale_is_dropped() {
    let snapshot = snapshot(
        &["en_US.UTF-8 UTF-8", "de_DE.UTF-8 UTF-8"],
        Some("en_US.utf8"),
    );

    let mut changes = ChangeSet::default();

    // A queue from disk never met the guard, so the change is queued whole.
    changes.want::<Locales>(locale_entry("en_US.UTF-8 UTF-8"), Toggle::Off, &snapshot);

    assert_eq!(changes.len(), 1);

    recheck(&mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn a_restored_switch_off_of_the_last_locale_is_dropped() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8"], None);

    let mut changes = ChangeSet::default();

    changes.want::<Locales>(locale_entry("en_US.UTF-8 UTF-8"), Toggle::Off, &snapshot);
    assert_eq!(changes.len(), 1);

    recheck(&mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn an_allowed_restored_switch_off_keeps_its_wish() {
    let snapshot = snapshot(
        &["en_US.UTF-8 UTF-8", "de_DE.UTF-8 UTF-8"],
        Some("en_US.utf8"),
    );

    let mut changes = ChangeSet::default();

    toggle(
        &locale_entry("de_DE.UTF-8 UTF-8"),
        false,
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert_eq!(recheck(&mut changes, &snapshot), Some(0));
    assert_eq!(changes.len(), 1);
}

#[test]
fn a_restored_switch_on_is_kept() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8"], None);
    let mut changes = ChangeSet::default();

    toggle(
        &locale_entry("de_DE.UTF-8 UTF-8"),
        true,
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(recheck(&mut changes, &snapshot), Some(0));
    assert_eq!(changes.len(), 1);
}
