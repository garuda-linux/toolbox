use std::collections::BTreeSet;

use toolbox_plan::procedure::Invocation;
use toolbox_plan::{
    FullName, HomeChange, HomeDirectory, HomeFolder, NewHome, PlannedAction, UserId,
};

use super::super::home::{Placement, check_uid, place, set_home};
use super::super::now::new_user;
use super::super::rows::user_rows;
use super::super::{Refusal, remove_user};
use super::{queued, snapshot};
use crate::domains::{Accounts, HomeFolders};
use crate::testing::user_name;
use crate::{ChangeSet, Snapshot, plan};

fn home(path: &str) -> HomeDirectory {
    HomeDirectory::new(path).unwrap()
}

fn uid(value: u32) -> UserId {
    UserId::new(value).unwrap()
}

fn change(path: &str, move_files: bool) -> HomeChange {
    HomeChange {
        home: home(path),
        move_files,
    }
}

fn placed_grace(placement: Placement, snapshot: &Snapshot) -> Result<Vec<String>, Refusal> {
    let user = new_user(
        user_name("grace"),
        FullName::default(),
        None,
        BTreeSet::new(),
        snapshot,
    )
    .unwrap();

    place(user, placement, snapshot).map(|user| {
        toolbox_plan::RootAction::CreateUser { user }
            .procedure()
            .iter()
            .map(ToString::to_string)
            .collect()
    })
}

#[test]
fn a_new_user_takes_the_defaults_unless_the_dialog_says_otherwise() {
    let snapshot = snapshot();

    assert_eq!(
        placed_grace(Placement::default(), &snapshot).unwrap(),
        ["useradd -m -- grace"]
    );
    assert_eq!(
        placed_grace(
            Placement {
                home: NewHome::At(home("/srv/grace")),
                uid: Some(uid(1500)),
                system: false,
            },
            &snapshot
        )
        .unwrap(),
        ["useradd -m -d /srv/grace -u 1500 -- grace"]
    );
    assert_eq!(
        placed_grace(
            Placement {
                home: NewHome::Absent,
                uid: Some(uid(450)),
                system: true,
            },
            &snapshot
        )
        .unwrap(),
        ["useradd -r -M -u 450 -- grace"]
    );
}

#[test]
fn a_user_id_must_be_free_and_fit_the_kind_of_account() {
    let snapshot = snapshot();

    for (asked, system, answer) in [
        (None, false, Ok(())),
        (None, true, Ok(())),
        (Some(uid(1500)), false, Ok(())),
        (Some(uid(450)), true, Ok(())),
        (Some(uid(1000)), false, Err(Refusal::UidInUse)),
        (Some(uid(1001)), false, Err(Refusal::UidInUse)),
        (Some(uid(33)), true, Err(Refusal::UidInUse)),
        (Some(uid(450)), false, Err(Refusal::UidOutOfRange)),
        (Some(uid(1500)), true, Err(Refusal::UidOutOfRange)),
    ] {
        assert_eq!(check_uid(asked, system, &snapshot), answer, "{asked:?}");
        assert_eq!(
            placed_grace(
                Placement {
                    home: NewHome::Standard,
                    uid: asked,
                    system,
                },
                &snapshot
            )
            .err(),
            answer.err(),
            "{asked:?}"
        );
    }

    assert_eq!(
        check_uid(Some(uid(1500)), false, &Snapshot::default()),
        Err(Refusal::Unavailable)
    );
}

#[test]
fn nobody_gets_the_home_of_another_account() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert_eq!(
        placed_grace(
            Placement {
                home: NewHome::At(home("/home/ada")),
                ..Placement::default()
            },
            &snapshot
        ),
        Err(Refusal::HomeTaken)
    );
    assert_eq!(
        set_home(
            &user_name("ada"),
            change("/home/nico", true),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::HomeTaken)
    );
    assert!(changes.is_empty());
}

#[test]
fn a_new_home_queues_with_its_choice_and_the_old_path_takes_it_back() {
    let snapshot = snapshot();
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    set_home(&ada, change("/srv/ada", true), &mut changes, &snapshot).unwrap();

    assert_eq!(queued(&changes), ["home-folders ada"]);
    assert_eq!(
        changes.wish::<HomeFolders>(&ada),
        Some(&change("/srv/ada", true))
    );

    let rows = user_rows(accounts, &snapshot, &changes, true);

    assert_eq!(
        rows.first().unwrap().queued.home,
        Some(change("/srv/ada", true))
    );
    assert!(!rows.first().unwrap().queued.is_empty());

    let shown: Vec<String> = plan(&changes)
        .steps()
        .iter()
        .flat_map(|step| match &step.action {
            PlannedAction::Root(action) => action.procedure(),
            PlannedAction::User(action) => action.procedure(),
        })
        .filter(|invocation| matches!(invocation, Invocation::Run(_)))
        .map(|invocation| invocation.to_string())
        .collect();

    assert_eq!(shown, ["usermod -d /srv/ada -m -- ada"]);

    set_home(&ada, change("/srv/ada", false), &mut changes, &snapshot).unwrap();
    assert_eq!(
        changes.wish::<HomeFolders>(&ada),
        Some(&change("/srv/ada", false))
    );

    set_home(&ada, change("/home/ada", true), &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn the_home_of_root_of_the_own_account_and_of_a_stranger_stays() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    for (user, refusal) in [
        ("root", Refusal::Root),
        ("toor", Refusal::Root),
        ("nico", Refusal::CurrentUser),
        ("ghost", Refusal::UnknownUser),
        ("http", Refusal::OutsideUserRange),
        ("nobody", Refusal::OutsideUserRange),
    ] {
        assert_eq!(
            set_home(
                &user_name(user),
                change("/srv/elsewhere", true),
                &mut changes,
                &snapshot
            ),
            Err(refusal),
            "{user}"
        );
    }

    assert_eq!(
        set_home(
            &user_name("ada"),
            change("/srv/ada", true),
            &mut changes,
            &Snapshot::default()
        ),
        Err(Refusal::Unavailable)
    );
    assert!(changes.is_empty());
}

#[test]
fn a_removed_user_takes_its_queued_home_change_along() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();
    let ada = user_name("ada");

    set_home(&ada, change("/srv/ada", true), &mut changes, &snapshot).unwrap();
    remove_user(&ada, HomeFolder::Keep, &mut changes, &snapshot).unwrap();

    assert_eq!(queued(&changes), ["removed-users ada"]);
}
