use std::collections::BTreeMap;

use toolbox_plan::procedure::Invocation;
use toolbox_plan::{FullName, HomeChange, HomeDirectory, HomeFolder, PlannedAction, ShellPath};

use super::{AccountLocks, FullNames, HomeFolders, NewGroups, RemovedGroups, RemovedUsers};
use crate::domain::{Queued, Summary, Toggle};
use crate::domains::{Accounts, CurrentShells, Groups, LoginShells};
use crate::test_support::{group, group_name, membership, restored, user, user_name, views};
use crate::{ChangeSet, Snapshot, SummaryCode, plan};

fn snapshot() -> Snapshot {
    let mut ada = user("ada", 1000, 1000);
    ada.gecos = toolbox_text::plain("Ada,Room 1");

    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![user("grace", 1001, 1001), ada],
        groups: vec![
            group("grace", 1001, &[]),
            group("makers", 1002, &["grace"]),
            group("ada", 1000, &[]),
            group("wheel", 998, &["ada"]),
        ],
    });
    snapshot.insert(CurrentShells(BTreeMap::from([
        (user_name("grace"), ShellPath::new("/usr/bin/bash").unwrap()),
        (user_name("ada"), ShellPath::new("/usr/bin/bash").unwrap()),
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
    changes.want::<RemovedUsers>(user_name("grace"), HomeFolder::Delete, &snapshot);
    changes.want::<AccountLocks>(user_name("ada"), Toggle::On, &snapshot);
    changes.want::<HomeFolders>(
        user_name("ada"),
        HomeChange {
            home: HomeDirectory::new("/srv/people/ada").unwrap(),
            move_files: true,
        },
        &snapshot,
    );
    changes.want::<FullNames>(user_name("ada"), FullName::new("A. L.").unwrap(), &snapshot);
    changes.want::<LoginShells>(
        user_name("grace"),
        ShellPath::new("/usr/bin/fish").unwrap(),
        &snapshot,
    );
    changes.want::<Groups>(membership("ada", "crafts"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("grace", "makers"), Toggle::Off, &snapshot);
    changes.want::<NewGroups>(group_name("crafts"), Queued, &snapshot);

    assert_eq!(
        shown(&changes),
        [
            "groupadd -- crafts",
            "gpasswd -a ada -- crafts",
            "gpasswd -d grace -- makers",
            "chsh -s /usr/bin/fish -- grace",
            "chfn -f 'A. L.' -- ada",
            "usermod -d /srv/people/ada -m -- ada",
            "usermod -L -- ada",
            "userdel -r -- grace",
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
    changes.want::<FullNames>(user_name("ada"), FullName::new("Ada").unwrap(), &snapshot);
    changes.want::<Groups>(membership("ada", "wheel"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("ada", "ada"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("grace", "wheel"), Toggle::Off, &snapshot);

    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn a_changed_full_name_compares_only_the_name_part() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<FullNames>(user_name("ada"), FullName::new("Ada").unwrap(), &snapshot);
    assert!(changes.is_empty(), "the name part is the name already");

    changes.want::<FullNames>(
        user_name("ada"),
        FullName::new("Ada Lovelace").unwrap(),
        &snapshot,
    );
    changes.want::<FullNames>(user_name("grace"), FullName::default(), &snapshot);
    changes.want::<FullNames>(user_name("ghost"), FullName::default(), &snapshot);

    assert_eq!(
        changes.len(),
        2,
        "grace has no name already, ghost is unknown"
    );
}

#[test]
fn locks_are_one_off_work_that_leaves_after_an_apply() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<AccountLocks>(user_name("grace"), Toggle::Off, &snapshot);
    changes.want::<AccountLocks>(user_name("ada"), Toggle::On, &snapshot);
    changes.want::<RemovedUsers>(user_name("grace"), HomeFolder::Keep, &snapshot);
    changes.reconcile(&snapshot);

    assert_eq!(changes.len(), 3, "no probe sees a lock");
    assert_eq!(
        shown(&changes),
        [
            "usermod -L -- ada",
            "usermod -U -- grace",
            "userdel -- grace"
        ]
    );

    crate::test_support::all_steps_succeeded(&mut changes);

    assert_eq!(shown(&changes), ["userdel -- grace"]);
}

#[test]
fn an_applied_removal_leaves_the_queue_with_the_next_snapshot() {
    let before = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedUsers>(user_name("grace"), HomeFolder::Keep, &before);
    changes.want::<RemovedGroups>(group_name("makers"), Queued, &before);
    changes.want::<NewGroups>(group_name("crafts"), Queued, &before);

    let mut after = Snapshot::default();
    after.insert(Accounts {
        users: vec![user("ada", 1000, 1000)],
        groups: vec![group("crafts", 1003, &[]), group("ada", 1000, &[])],
    });
    changes.reconcile(&after);

    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn the_review_page_gets_keys_and_summaries() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedUsers>(user_name("grace"), HomeFolder::Delete, &snapshot);
    changes.want::<AccountLocks>(user_name("ada"), Toggle::Off, &snapshot);
    changes.want::<Groups>(membership("grace", "wheel"), Toggle::On, &snapshot);
    changes.want::<FullNames>(
        user_name("grace"),
        FullName::new("Grace").unwrap(),
        &snapshot,
    );

    assert_eq!(
        views(&changes),
        [
            (
                "groups",
                "grace wheel".to_owned(),
                Summary::pair(SummaryCode::GroupJoin, "grace", "wheel")
            ),
            (
                "full-names",
                "grace".to_owned(),
                Summary::pair(SummaryCode::FullNameSet, "grace", "Grace")
            ),
            (
                "account-locks",
                "ada".to_owned(),
                Summary::of(SummaryCode::AccountUnlock, "ada")
            ),
            (
                "removed-users",
                "grace".to_owned(),
                Summary::of(SummaryCode::UserRemoveWithHome, "grace")
            ),
        ]
    );

    assert!(changes.remove_key("removed-users", "grace"));
    assert!(changes.remove_key("groups", "grace wheel"));
    assert_eq!(changes.len(), 2);
}

#[test]
fn queued_account_changes_survive_a_restart() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    changes.want::<RemovedUsers>(user_name("grace"), HomeFolder::Delete, &snapshot);
    changes.want::<AccountLocks>(user_name("ada"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("grace", "wheel"), Toggle::On, &snapshot);
    changes.want::<FullNames>(
        user_name("grace"),
        FullName::new("Grace").unwrap(),
        &snapshot,
    );
    changes.want::<NewGroups>(group_name("crafts"), Queued, &snapshot);
    changes.want::<RemovedGroups>(group_name("makers"), Queued, &snapshot);
    changes.want::<LoginShells>(
        user_name("grace"),
        ShellPath::new("/usr/bin/fish").unwrap(),
        &snapshot,
    );

    let (restored, problems) = restored(&changes, &snapshot);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
    assert_eq!(restored.len(), 7);
}
