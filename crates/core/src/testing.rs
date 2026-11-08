//! Fixtures shared by the tests of this crate.

use std::collections::BTreeSet;

use serde_json::Value;
use toolbox_plan::procedure::Invocation;
use toolbox_plan::{
    GroupName, LocaleEntry, LocaleName, PackageName, Plan, PlannedAction, PlymouthTheme, UnitName,
    UserName,
};

use crate::domains::{AccountLockFlags, Accounts, Group, Membership, User};
use crate::{
    ChangeSet, RestoreProblem, SAVED_VERSION, SavedChange, SavedChanges, Snapshot, Summary,
    domains, plan,
};

/// The user id of `nobody`, above the ids of people.
pub const NOBODY_ID: u32 = 0xFFFE;

pub fn user_name(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

pub fn group_name(name: &str) -> GroupName {
    GroupName::new(name).unwrap()
}

pub fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap()
}

pub fn unit(name: &str) -> UnitName {
    UnitName::new(name).unwrap()
}

pub fn packages(names: &[&str]) -> BTreeSet<PackageName> {
    names.iter().map(|name| package(name)).collect()
}

pub fn locale(name: &str) -> LocaleName {
    LocaleName::new(name).unwrap()
}

pub fn locale_entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

pub fn plymouth_theme(name: &str) -> PlymouthTheme {
    PlymouthTheme::new(name).unwrap()
}

pub fn membership(user: &str, group: &str) -> Membership {
    Membership::new(user_name(user), group_name(group))
}

pub fn lock_flags(flags: &[(&str, bool)]) -> AccountLockFlags {
    AccountLockFlags(
        flags
            .iter()
            .map(|(name, locked)| (user_name(name), *locked))
            .collect(),
    )
}

/// A user with the home and shell useradd would give it.
pub fn user(name: &str, uid: u32, gid: u32) -> User {
    User {
        name: user_name(name),
        uid,
        gid,
        gecos: String::new(),
        home: format!("/home/{name}"),
        shell: "/usr/bin/bash".to_owned(),
    }
}

pub fn group(name: &str, gid: u32, members: &[&str]) -> Group {
    Group {
        name: group_name(name),
        gid,
        members: members.iter().map(|member| user_name(member)).collect(),
    }
}

/// `nico` (1000, own group) listed in each of `groups`, which get the
/// system ids 900 and up.
pub fn nina_in(groups: &[&str]) -> Accounts {
    let mut all = vec![group("nico", 1000, &[])];

    all.extend(
        (900_u32..)
            .zip(groups)
            .map(|(gid, name)| group(name, gid, &["nico"])),
    );
    all.sort_by(|left, right| left.name.cmp(&right.name));

    Accounts {
        users: vec![user("nico", 1000, 1000)],
        groups: all,
    }
}

pub fn actions(changes: &ChangeSet) -> Vec<PlannedAction> {
    plan(changes)
        .steps()
        .iter()
        .map(|step| step.action.clone())
        .collect()
}

fn procedure(action: &PlannedAction) -> Vec<Invocation> {
    match action {
        PlannedAction::Root(action) => action.procedure(),
        PlannedAction::User(action) => action.procedure(),
    }
}

pub fn commands(plan: &Plan) -> Vec<String> {
    plan.steps()
        .iter()
        .flat_map(|step| procedure(&step.action))
        .map(|invocation| invocation.to_string())
        .collect()
}

pub fn sided(changes: &ChangeSet) -> Vec<String> {
    plan(changes)
        .steps()
        .iter()
        .map(|step| {
            let side = match step.action {
                PlannedAction::Root(_) => "root",
                PlannedAction::User(_) => "user",
            };

            let commands: Vec<String> = procedure(&step.action)
                .iter()
                .map(Invocation::to_string)
                .collect();

            format!("{side}: {}", commands.join(" && "))
        })
        .collect()
}

pub fn views(changes: &ChangeSet) -> Vec<(&'static str, String, Summary)> {
    changes
        .views()
        .into_iter()
        .map(|view| (view.domain, view.key, view.summary))
        .collect()
}

/// The queue after a trip through the JSON file and back with every domain.
pub fn restored(changes: &ChangeSet, snapshot: &Snapshot) -> (ChangeSet, Vec<RestoreProblem>) {
    let json = serde_json::to_string(&changes.to_saved()).unwrap();
    let mut restored = ChangeSet::default();
    let problems = domains::all().unwrap().restore(
        &mut restored,
        &serde_json::from_str(&json).unwrap(),
        snapshot,
    );

    (restored, problems)
}

/// A queue restored from a file holding `saved`, each change as domain,
/// item and wish. Every change must come back.
pub fn restore_saved(saved: &[(&str, Value, Value)]) -> ChangeSet {
    let saved = SavedChanges {
        version: SAVED_VERSION,
        changes: saved
            .iter()
            .map(|(domain, item, wish)| SavedChange {
                domain: (*domain).to_owned(),
                item: item.clone(),
                wish: wish.clone(),
            })
            .collect(),
    };

    let mut changes = ChangeSet::default();
    let problems = domains::all()
        .unwrap()
        .restore(&mut changes, &saved, &Snapshot::default());

    assert_eq!(problems, []);
    changes
}

/// The queue after an apply in which every step succeeded.
pub fn all_steps_succeeded(changes: &mut ChangeSet) {
    let done: Vec<PlannedAction> = plan(changes)
        .steps()
        .iter()
        .map(|step| step.action.clone())
        .collect();

    changes.drop_done_one_offs(&done);
}
