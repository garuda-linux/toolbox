//! A restored blacklist change meets the Drivers tab's guard once the
//! module facts are part of the snapshot.

use super::drivers::{BROADCOM_FILE, OWN_FILE, entry, facts, module};
use crate::domains::ModuleBlacklist;
use crate::features::network::{
    DriverDetail, DriverRow, ModuleFacts, driver_rows, recheck_blacklist, set_blacklisted,
};
use crate::{ChangeSet, Snapshot, Toggle};

fn driver<'rows>(rows: &'rows [DriverRow], name: &str) -> &'rows DriverRow {
    rows.iter().find(|row| row.name.as_str() == name).unwrap()
}

#[test]
fn the_blacklist_item_asks_for_the_change_the_row_does_not_show() {
    let facts = facts(vec![entry("r8169", OWN_FILE), entry("b43", BROADCOM_FILE)]);
    let mut changes = ChangeSet::default();

    set_blacklisted(
        &module("mt76-usb"),
        Toggle::On,
        &facts,
        &mut changes,
        &Snapshot::default(),
    )
    .unwrap();

    let rows = driver_rows(&facts, &changes);
    let action = |name: &str| {
        driver(&rows, name)
            .blacklist_action()
            .map(|action| (action.text_id, action.wish))
    };

    assert_eq!(
        action("8139too"),
        Some(("network-driver-blacklist", Toggle::On))
    );
    assert_eq!(action("r8169"), Some(("network-driver-allow", Toggle::Off)));
    assert_eq!(
        action("mt76_usb"),
        Some(("network-driver-unqueue", Toggle::Off))
    );
    assert_eq!(action("b43"), None, "only another file lists it");
}

#[test]
fn the_queued_change_is_the_detail_before_the_other_files() {
    let facts = facts(vec![entry("b43", BROADCOM_FILE)]);
    let mut changes = ChangeSet::default();

    set_blacklisted(
        &module("mt76-usb"),
        Toggle::On,
        &facts,
        &mut changes,
        &Snapshot::default(),
    )
    .unwrap();

    let rows = driver_rows(&facts, &changes);
    let files = [toolbox_text::plain(BROADCOM_FILE)];

    assert_eq!(
        driver(&rows, "mt76_usb").detail(),
        Some(DriverDetail::Queued(Toggle::On))
    );
    assert_eq!(
        driver(&rows, "b43").detail(),
        Some(DriverDetail::OtherFiles(&files))
    );
    assert_eq!(driver(&rows, "8139too").detail(), None);
}

fn with_facts(facts: ModuleFacts) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(facts);
    snapshot
}

#[test]
fn a_restored_blacklist_of_a_module_only_another_file_lists_is_dropped() {
    let state = with_facts(facts(vec![entry("b43", BROADCOM_FILE)]));
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("b43"), Toggle::On, &Snapshot::default());

    assert_eq!(recheck_blacklist(&mut changes, &state), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_blacklist_of_an_own_entry_keeps_its_wish() {
    let state = with_facts(facts(vec![entry("b43", OWN_FILE)]));
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("b43"), Toggle::On, &Snapshot::default());

    assert_eq!(recheck_blacklist(&mut changes, &state), Some(0));
    assert_eq!(changes.len(), 1);
}

#[test]
fn a_restored_blacklist_change_is_rescanned_with_the_facts_it_has() {
    let state = with_facts(facts(vec![entry("b43", BROADCOM_FILE)]));
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("b43"), Toggle::Off, &Snapshot::default());

    assert_eq!(recheck_blacklist(&mut changes, &state), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_blacklist_change_waits_for_the_module_facts() {
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("b43"), Toggle::Off, &Snapshot::default());

    assert_eq!(recheck_blacklist(&mut changes, &Snapshot::default()), None);
    assert_eq!(changes.len(), 1);
}

#[test]
fn a_restored_dash_spelling_is_one_item_with_the_underscore_one_and_is_dropped() {
    let state = with_facts(facts(vec![entry("iwl_wifi", BROADCOM_FILE)]));
    let saved = crate::SavedChanges {
        version: crate::SAVED_VERSION,
        changes: vec![crate::SavedChange {
            domain: "module-blacklist".to_owned(),
            item: serde_json::json!("iwl-wifi"),
            wish: serde_json::json!("on"),
            outcome: crate::Outcome::Untried,
        }],
    };

    let mut changes = ChangeSet::default();

    crate::domains::all()
        .unwrap()
        .restore(&mut changes, &saved, &Snapshot::default());
    changes.want::<ModuleBlacklist>(module("iwl_wifi"), Toggle::On, &Snapshot::default());

    assert_eq!(changes.len(), 1);
    assert_eq!(recheck_blacklist(&mut changes, &state), Some(1));
    assert!(changes.is_empty());
}
