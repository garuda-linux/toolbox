//! Restored memberships and group removals meet the page's guards once
//! the account files are known.

use serde_json::json;

use super::super::recheck_groups;
use super::{queued, snapshot};
use crate::Snapshot;
use crate::testing::restore_saved;

#[test]
fn a_restored_membership_the_page_refuses_is_dropped() {
    let mut changes = restore_saved(&[
        (
            "groups",
            json!({ "user": "toor", "group": "wheel" }),
            json!("on"),
        ),
        (
            "groups",
            json!({ "user": "ghost", "group": "wheel" }),
            json!("on"),
        ),
        (
            "groups",
            json!({ "user": "ada", "group": "ada" }),
            json!("off"),
        ),
        (
            "groups",
            json!({ "user": "ada", "group": "wheel" }),
            json!("on"),
        ),
    ]);

    assert_eq!(recheck_groups(&mut changes, &snapshot()), Some(3));
    assert_eq!(queued(&changes), ["groups ada wheel"]);
}

/// A catalog toggle may queue a group its package creates.
#[test]
fn a_restored_membership_of_a_group_still_to_come_stays() {
    let mut changes = restore_saved(&[(
        "groups",
        json!({ "user": "nico", "group": "docker" }),
        json!("on"),
    )]);

    assert_eq!(recheck_groups(&mut changes, &snapshot()), Some(0));
    assert_eq!(queued(&changes), ["groups nico docker"]);
}

#[test]
fn a_restored_removal_of_a_system_or_primary_group_is_dropped() {
    let mut changes = restore_saved(&[
        ("removed-groups", json!("http"), json!(null)),
        ("removed-groups", json!("ada"), json!(null)),
        ("removed-groups", json!("makers"), json!(null)),
    ]);

    assert_eq!(recheck_groups(&mut changes, &snapshot()), Some(2));
    assert_eq!(queued(&changes), ["removed-groups makers"]);
}

#[test]
fn without_the_account_files_the_restored_groups_wait() {
    let mut changes = restore_saved(&[("removed-groups", json!("http"), json!(null))]);

    assert_eq!(recheck_groups(&mut changes, &Snapshot::default()), None);
    assert_eq!(changes.len(), 1);
}
