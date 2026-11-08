use std::collections::BTreeSet;

use serde_json::json;

use super::{Domains, DuplicateDomain, RestoreProblem, SAVED_VERSION, SavedChange, SavedChanges};
use crate::domain::Toggle;
use crate::domains::{Groups, InstalledPackages, NewGroups, Packages};
use crate::testing::{membership, package};
use crate::{ChangeSet, Queued, Snapshot};

fn domains() -> Domains {
    Domains::default()
        .with::<Packages>()
        .and_then(Domains::with::<Groups>)
        .unwrap()
}

#[test]
fn queued_changes_survive_a_save_and_restore() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("mpv"), Toggle::On, &Snapshot::default());
    changes.want::<Groups>(
        membership("nico", "docker"),
        Toggle::Off,
        &Snapshot::default(),
    );

    let json = serde_json::to_string(&changes.to_saved()).unwrap();
    let saved: SavedChanges = serde_json::from_str(&json).unwrap();
    let mut restored = ChangeSet::default();
    let problems = domains().restore(&mut restored, &saved, &Snapshot::default());

    assert_eq!(problems, Vec::<RestoreProblem>::new());
    assert_eq!(restored.views(), changes.views());
}

#[test]
fn the_saved_form_is_plain_json() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("mpv"), Toggle::On, &Snapshot::default());

    assert_eq!(
        serde_json::to_value(changes.to_saved()).unwrap(),
        json!({
            "version": SAVED_VERSION,
            "changes": [{ "domain": "packages", "item": "mpv", "wish": "on" }],
        })
    );
}

#[test]
fn a_change_the_system_meanwhile_fulfils_is_not_restored() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("mpv"), Toggle::On, &Snapshot::default());
    let saved = changes.to_saved();
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("mpv")])));

    let mut restored = ChangeSet::default();
    domains().restore(&mut restored, &saved, &snapshot);

    assert!(restored.is_empty());
}

#[test]
fn unknown_domains_and_invalid_items_are_skipped_and_reported() {
    let saved = SavedChanges {
        version: SAVED_VERSION,
        changes: vec![
            SavedChange {
                domain: "kernels".to_owned(),
                item: json!("linux"),
                wish: json!("on"),
            },
            SavedChange {
                domain: "packages".to_owned(),
                item: json!("-Syu"),
                wish: json!("on"),
            },
            SavedChange {
                domain: "packages".to_owned(),
                item: json!("vlc"),
                wish: json!("on"),
            },
        ],
    };

    let mut restored = ChangeSet::default();

    let problems = domains().restore(&mut restored, &saved, &Snapshot::default());

    assert_eq!(restored.len(), 1);
    assert_eq!(problems.len(), 2);
    assert_eq!(
        problems[0],
        RestoreProblem::UnknownDomain("kernels".to_owned())
    );
}

#[test]
fn a_file_of_another_version_restores_nothing() {
    let saved = SavedChanges {
        version: SAVED_VERSION + 1,
        changes: Vec::new(),
    };

    let mut restored = ChangeSet::default();

    let problems = domains().restore(&mut restored, &saved, &Snapshot::default());

    assert_eq!(problems, [RestoreProblem::Version(SAVED_VERSION + 1)]);
}

#[test]
fn a_second_domain_with_the_same_id_is_refused() {
    let refused = domains().with::<Packages>().err();

    assert_eq!(refused, Some(DuplicateDomain("packages")));
    assert_eq!(
        refused.map(|problem| problem.to_string()),
        Some("two domains share the id `packages`".to_owned())
    );
}

#[test]
fn work_to_do_is_saved_without_a_wish() {
    let mut changes = ChangeSet::default();
    changes.want::<NewGroups>(
        crate::testing::group_name("crafts"),
        Queued,
        &Snapshot::default(),
    );

    assert_eq!(
        serde_json::to_value(changes.to_saved()).unwrap(),
        json!({
            "version": SAVED_VERSION,
            "changes": [{ "domain": "new-groups", "item": "crafts", "wish": null }],
        })
    );
}
