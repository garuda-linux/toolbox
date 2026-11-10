use super::super::{Refusal, add_group, remove_group, set_groups};
use super::{groups, queued, snapshot};
use crate::ChangeSet;
use crate::domains::{Groups, NewGroups, RemovedGroups};
use crate::test_support::{group_name, membership, user_name};

#[test]
fn groups_are_added_once_and_system_or_primary_groups_stay() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert_eq!(
        add_group(&group_name("wheel"), &mut changes, &snapshot),
        Err(Refusal::GroupExists)
    );
    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();
    assert_eq!(
        add_group(&group_name("crafts"), &mut changes, &snapshot),
        Err(Refusal::GroupExists)
    );

    for (name, refusal) in [
        ("wheel", Refusal::SystemGroup),
        ("audio", Refusal::SystemGroup),
        ("nico", Refusal::PrimaryGroup),
        ("ghosts", Refusal::UnknownGroup),
    ] {
        assert_eq!(
            remove_group(&group_name(name), &mut changes, &snapshot),
            Err(refusal),
            "{name}"
        );
    }

    assert_eq!(queued(&changes), ["new-groups crafts"]);
}

#[test]
fn removing_a_group_drops_its_queued_memberships_and_can_be_taken_back() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let makers = group_name("makers");

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();
    set_groups(
        &user_name("ada"),
        &groups(&["audio", "crafts"]),
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(
        queued(&changes),
        [
            "new-groups crafts",
            "groups ada crafts",
            "groups ada makers"
        ]
    );

    remove_group(&makers, &mut changes, &snapshot).unwrap();
    assert_eq!(
        queued(&changes),
        [
            "new-groups crafts",
            "groups ada crafts",
            "removed-groups makers"
        ]
    );
    assert_eq!(
        set_groups(
            &user_name("ada"),
            &groups(&["makers"]),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::UnknownGroup),
        "a group that is about to go takes no members"
    );

    add_group(&makers, &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<RemovedGroups>(&makers), None);

    remove_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<NewGroups>(&group_name("crafts")), None);
    assert_eq!(changes.wish::<Groups>(&membership("ada", "crafts")), None);
    assert!(changes.is_empty(), "{changes:?}");
}
