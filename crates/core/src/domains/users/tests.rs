use std::collections::BTreeMap;

use toolbox_plan::procedure::Invocation;
use toolbox_plan::{FullName, HomeChange, HomeDirectory, HomeFolder, PlannedAction, ShellPath};

use super::{AccountLocks, FullNames, HomeFolders, NewGroups, RemovedGroups, RemovedUsers};
use crate::domain::{Queued, Summary, Toggle};
use crate::domains::{Accounts, CurrentShells, Groups, LoginShells};
use crate::testing::{group, group_name, membership, restored, user, user_name, views};
use crate::{ChangeSet, Snapshot, plan};

fn snapshot() -> Snapshot {
    let mut nico = user("nico", 1000, 1000);
    nico.gecos = "Nico,Room 1".to_owned();

    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![user("ada", 1001, 1001), nico],
        groups: vec![
            group("ada", 1001, &[]),
            group("makers", 1002, &["ada"]),
            group("nico", 1000, &[]),
            group("wheel", 998, &["nico"]),
        ],
    });
    snapshot.insert(CurrentShells(BTreeMap::from([
        (user_name("ada"), ShellPath::new("/usr/bin/bash").unwrap()),
        (user_name("nico"), ShellPath::new("/usr/bin/bash").unwrap()),
    ])));
    snapshot
}

fn shown(changes: &ChangeSet) -> Vec<String> {
    plan(changes)
        .steps()
        .iter()
        .flat_map(|step| match &step.action {
            PlannedAction::Root(action) => action.procedure(),
            PlannedAction::User(action) => action.procedure(),
        })
        .filter(|invocation| matches!(invocation, Invocation::Run(_)))
        .map(|invocation| invocation.to_string())
        .collect()
}

#[test]
fn account_changes_run_in_an_order_that_works() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedGroups>(group_name("makers"), Queued, &snapshot);
    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Delete, &snapshot);
    changes.want::<AccountLocks>(user_name("nico"), Toggle::On, &snapshot);
    changes.want::<HomeFolders>(
        user_name("nico"),
        HomeChange {
            home: HomeDirectory::new("/srv/people/nico").unwrap(),
            move_files: true,
        },
        &snapshot,
    );
    changes.want::<FullNames>(
        user_name("nico"),
        FullName::new("N. J.").unwrap(),
        &snapshot,
    );
    changes.want::<LoginShells>(
        user_name("ada"),
        ShellPath::new("/usr/bin/fish").unwrap(),
        &snapshot,
    );
    changes.want::<Groups>(membership("nico", "crafts"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("ada", "makers"), Toggle::Off, &snapshot);
    changes.want::<NewGroups>(group_name("crafts"), Queued, &snapshot);

    assert_eq!(
        shown(&changes),
        [
            "groupadd -- crafts",
            "gpasswd -d ada makers",
            "gpasswd -a nico crafts",
            "chsh -s /usr/bin/fish -- ada",
            "chfn -f 'N. J.' -- nico",
            "usermod -d /srv/people/nico -m -- nico",
            "usermod -L -- nico",
            "userdel -r -- ada",
            "groupdel -- makers",
        ]
    );
}

#[test]
fn wishes_the_system_fulfils_drop_out() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<NewGroups>(group_name("wheel"), Queued, &snapshot);
    changes.want::<RemovedGroups>(group_name("ghosts"), Queued, &snapshot);
    changes.want::<RemovedUsers>(user_name("ghost"), HomeFolder::Keep, &snapshot);
    changes.want::<FullNames>(user_name("nico"), FullName::new("Nico").unwrap(), &snapshot);
    changes.want::<Groups>(membership("nico", "wheel"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("nico", "nico"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("ada", "wheel"), Toggle::Off, &snapshot);

    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn a_changed_full_name_compares_only_the_name_part() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<FullNames>(
        user_name("nico"),
        FullName::new("Nico Jensch").unwrap(),
        &snapshot,
    );
    changes.want::<FullNames>(user_name("ada"), FullName::default(), &snapshot);
    changes.want::<FullNames>(user_name("ghost"), FullName::default(), &snapshot);

    assert_eq!(
        changes.len(),
        2,
        "ada has no name already, ghost is unknown"
    );
}

#[test]
fn locks_are_one_off_work_that_leaves_after_an_apply() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<AccountLocks>(user_name("ada"), Toggle::Off, &snapshot);
    changes.want::<AccountLocks>(user_name("nico"), Toggle::On, &snapshot);
    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Keep, &snapshot);
    changes.reconcile(&snapshot);

    assert_eq!(changes.len(), 3, "no probe sees a lock");
    assert_eq!(
        shown(&changes),
        ["usermod -U -- ada", "usermod -L -- nico", "userdel -- ada"]
    );

    crate::testing::all_steps_succeeded(&mut changes);

    assert_eq!(shown(&changes), ["userdel -- ada"]);
}

#[test]
fn an_applied_removal_leaves_the_queue_with_the_next_snapshot() {
    let before = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Keep, &before);
    changes.want::<RemovedGroups>(group_name("makers"), Queued, &before);
    changes.want::<NewGroups>(group_name("crafts"), Queued, &before);

    let mut after = Snapshot::default();
    after.insert(Accounts {
        users: vec![user("nico", 1000, 1000)],
        groups: vec![group("crafts", 1003, &[]), group("nico", 1000, &[])],
    });
    changes.reconcile(&after);

    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn the_review_page_gets_keys_and_summaries() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Delete, &snapshot);
    changes.want::<AccountLocks>(user_name("nico"), Toggle::Off, &snapshot);
    changes.want::<Groups>(membership("ada", "wheel"), Toggle::On, &snapshot);
    changes.want::<FullNames>(user_name("ada"), FullName::new("Ada").unwrap(), &snapshot);

    assert_eq!(
        views(&changes),
        [
            (
                "groups",
                "ada wheel".to_owned(),
                Summary::pair("group-join", "ada", "wheel")
            ),
            (
                "full-names",
                "ada".to_owned(),
                Summary::pair("full-name-set", "ada", "Ada")
            ),
            (
                "account-locks",
                "nico".to_owned(),
                Summary::of("account-unlock", "nico")
            ),
            (
                "removed-users",
                "ada".to_owned(),
                Summary::of("user-remove-with-home", "ada")
            ),
        ]
    );

    assert!(changes.remove_key("removed-users", "ada"));
    assert!(changes.remove_key("groups", "ada wheel"));
    assert_eq!(changes.len(), 2);
}

#[test]
fn queued_account_changes_survive_a_restart() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedUsers>(user_name("ada"), HomeFolder::Delete, &snapshot);
    changes.want::<AccountLocks>(user_name("nico"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("ada", "wheel"), Toggle::On, &snapshot);
    changes.want::<FullNames>(user_name("ada"), FullName::new("Ada").unwrap(), &snapshot);
    changes.want::<NewGroups>(group_name("crafts"), Queued, &snapshot);
    changes.want::<RemovedGroups>(group_name("makers"), Queued, &snapshot);
    changes.want::<LoginShells>(
        user_name("ada"),
        ShellPath::new("/usr/bin/fish").unwrap(),
        &snapshot,
    );

    let (restored, problems) = restored(&changes, &snapshot);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
    assert_eq!(restored.len(), 7);
}
