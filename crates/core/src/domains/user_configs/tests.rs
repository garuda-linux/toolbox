use std::collections::BTreeMap;

use toolbox_plan::{
    ConfigReload, ConfigWord, HomePath, PlannedAction, TargetList, UnitName, UnitOperation,
    UserAction, UserConfigEdit,
};

use super::{UserConfigStates, UserConfigWish, UserConfigs};
use crate::domain::{Summary, Toggle};
use crate::testing::{actions, failed_at, outcomes, restored, stopped_at, unit, views};
use crate::{ChangeSet, Snapshot, plan};

fn audio() -> UserConfigEdit {
    UserConfigEdit::File {
        path: HomePath::new(".config/wireplumber/wireplumber.conf.d/disable-suspension.conf")
            .unwrap(),
        content: "x\n".to_owned(),
    }
}

fn discord() -> UserConfigEdit {
    UserConfigEdit::JsonFlag {
        path: HomePath::new(".config/discord/settings.json").unwrap(),
        key: ConfigWord::new("SKIP_HOST_UPDATE").unwrap(),
    }
}

fn other_audio() -> UserConfigEdit {
    UserConfigEdit::File {
        path: HomePath::new(".config/pipewire/x.conf").unwrap(),
        content: "y\n".to_owned(),
    }
}

fn units(edit: &UserConfigEdit) -> Vec<UnitName> {
    if *edit == audio() {
        vec![unit("wireplumber.service"), unit("pipewire.service")]
    } else if *edit == other_audio() {
        vec![unit("pipewire.service")]
    } else {
        Vec::new()
    }
}

fn want(changes: &mut ChangeSet, edit: UserConfigEdit, toggle: Toggle, snapshot: &Snapshot) {
    let wish = UserConfigWish::new(toggle, units(&edit));

    changes.want::<UserConfigs>(edit, wish, snapshot);
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(UserConfigStates(BTreeMap::from([
        (audio(), false),
        (discord(), true),
    ])));
    snapshot
}

#[test]
fn an_edit_as_it_is_drops_out_and_an_unknown_one_stays() {
    let mut changes = ChangeSet::default();
    want(&mut changes, discord(), Toggle::On, &snapshot());
    want(&mut changes, audio(), Toggle::Off, &snapshot());

    assert!(changes.is_empty());

    want(&mut changes, other_audio(), Toggle::Off, &snapshot());

    assert_eq!(changes.len(), 1);
}

#[test]
fn edits_run_first_then_one_restart_of_every_unit() {
    let mut changes = ChangeSet::default();
    want(&mut changes, audio(), Toggle::On, &snapshot());
    want(&mut changes, other_audio(), Toggle::On, &snapshot());
    want(&mut changes, discord(), Toggle::Off, &snapshot());

    let actions = actions(&changes);

    assert_eq!(
        actions,
        [
            PlannedAction::User(UserAction::EditUserConfig {
                edit: other_audio(),
                on: true,
            }),
            PlannedAction::User(UserAction::EditUserConfig {
                edit: audio(),
                on: true,
            }),
            PlannedAction::User(UserAction::EditUserConfig {
                edit: discord(),
                on: false,
            }),
            PlannedAction::User(UserAction::ControlUserUnits {
                operation: UnitOperation::Restart,
                units: TargetList::new([unit("pipewire.service"), unit("wireplumber.service")])
                    .unwrap(),
            }),
        ]
    );
}

#[test]
fn without_units_there_is_no_restart() {
    let mut changes = ChangeSet::default();
    want(&mut changes, discord(), Toggle::Off, &snapshot());

    assert_eq!(plan(&changes).steps().len(), 1);
}

#[test]
fn the_review_key_names_file_and_key() {
    let mut changes = ChangeSet::default();
    want(&mut changes, discord(), Toggle::Off, &snapshot());

    assert_eq!(
        views(&changes),
        [(
            "user-configs",
            "~/.config/discord/settings.json SKIP_HOST_UPDATE".to_owned(),
            Summary::of(
                "user-config-off",
                "~/.config/discord/settings.json SKIP_HOST_UPDATE"
            )
        )]
    );
    assert!(changes.remove_key(
        "user-configs",
        "~/.config/discord/settings.json SKIP_HOST_UPDATE"
    ));
    assert!(changes.is_empty());
}

#[test]
fn queued_edits_survive_a_save_and_restore() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    want(&mut changes, audio(), Toggle::On, &empty);
    want(&mut changes, discord(), Toggle::Off, &empty);

    let (restored, problems) = restored(&changes, &empty);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
    assert_eq!(
        restored.wish::<UserConfigs>(&audio()),
        Some(&UserConfigWish::new(Toggle::On, units(&audio())))
    );
}

#[test]
fn a_reload_runs_once_after_the_edits() {
    let kwin = |key: &str| UserConfigEdit::IniKey {
        path: HomePath::new(".config/kwinrc").unwrap(),
        group: ConfigWord::new("Windows").unwrap(),
        key: ConfigWord::new(key).unwrap(),
        on: ConfigWord::new("true").unwrap(),
        off: ConfigWord::new("false").unwrap(),
    };

    let wish = UserConfigWish::new(Toggle::On, []).with_reload(Some(ConfigReload::Kwin));
    let mut changes = ChangeSet::default();

    changes.want::<UserConfigs>(
        kwin("BorderlessMaximizedWindows"),
        wish.clone(),
        &snapshot(),
    );
    changes.want::<UserConfigs>(kwin("Other"), wish, &snapshot());

    let actions = actions(&changes);

    assert_eq!(actions.len(), 3);
    assert_eq!(
        actions.last(),
        Some(&PlannedAction::User(UserAction::ReloadConfig {
            target: ConfigReload::Kwin
        }))
    );
}

#[test]
fn an_edit_is_one_item_whatever_units_come_with_it() {
    let first = UserConfigWish::new(Toggle::On, [unit("wireplumber.service")]);
    let second = UserConfigWish::new(Toggle::On, [unit("pipewire.service")]);
    let mut changes = ChangeSet::default();

    changes.want::<UserConfigs>(audio(), first, &snapshot());
    changes.want::<UserConfigs>(audio(), second.clone(), &snapshot());

    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes.wish::<UserConfigs>(&audio()),
        Some(&second),
        "the units of the later wish count"
    );
}

/// The file is written, but the units that read it were never restarted.
#[test]
fn a_failed_restart_keeps_the_edit_queued() {
    let mut changes = ChangeSet::default();
    want(&mut changes, audio(), Toggle::On, &snapshot());

    stopped_at(&mut changes, 2);
    let mut written = Snapshot::default();
    written.insert(UserConfigStates(BTreeMap::from([
        (audio(), true),
        (discord(), true),
    ])));
    changes.reconcile(&written);

    assert_eq!(
        outcomes(&changes)
            .into_iter()
            .map(|(_, outcome)| outcome)
            .collect::<Vec<_>>(),
        [failed_at(2)]
    );
}
