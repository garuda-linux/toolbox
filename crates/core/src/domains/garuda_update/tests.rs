use std::collections::BTreeSet;

use toolbox_plan::{
    DnsProvider, GarudaUpdateFlag, PlannedAction, RootAction, ShellPath, WifiBackend,
};

use super::{GarudaUpdateConfig, GarudaUpdateFlags};
use crate::domain::{Summary, Toggle};
use crate::domains::{Dns, Hblock, LoginShells, WifiBackends};
use crate::test_support::{actions, restored, user_name};
use crate::{ChangeSet, Single, Snapshot, SummaryCode};

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(GarudaUpdateConfig(BTreeSet::from([
        GarudaUpdateFlag::UpdateAur,
    ])));
    snapshot
}

#[test]
fn a_switch_as_it_is_drops_out() {
    let mut changes = ChangeSet::default();
    changes.want::<GarudaUpdateFlags>(GarudaUpdateFlag::UpdateAur, Toggle::On, &snapshot());
    changes.want::<GarudaUpdateFlags>(GarudaUpdateFlag::NoCheckSpace, Toggle::Off, &snapshot());

    assert!(changes.is_empty());
}

#[test]
fn each_switch_is_its_own_step() {
    let mut changes = ChangeSet::default();
    changes.want::<GarudaUpdateFlags>(GarudaUpdateFlag::UpdateAur, Toggle::Off, &snapshot());
    changes.want::<GarudaUpdateFlags>(GarudaUpdateFlag::SkipMirrorlist, Toggle::On, &snapshot());

    let actions = actions(&changes);

    assert_eq!(
        actions,
        [
            PlannedAction::Root(RootAction::SetGarudaUpdateFlag {
                flag: GarudaUpdateFlag::SkipMirrorlist,
                enabled: true,
            }),
            PlannedAction::Root(RootAction::SetGarudaUpdateFlag {
                flag: GarudaUpdateFlag::UpdateAur,
                enabled: false,
            }),
        ]
    );

    let keys: Vec<(String, Summary)> = changes
        .views()
        .into_iter()
        .map(|view| (view.key, view.summary))
        .collect();

    assert_eq!(
        keys,
        [
            (
                "skip-mirrorlist".to_owned(),
                Summary::of(SummaryCode::GarudaUpdateFlagOn, "skip-mirrorlist")
            ),
            (
                "update-aur".to_owned(),
                Summary::of(SummaryCode::GarudaUpdateFlagOff, "update-aur")
            ),
        ]
    );
}

/// Every settings domain is registered, so its queued changes survive a
/// restart.
#[test]
fn settings_changes_survive_a_save_and_restore() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<LoginShells>(
        user_name("nico"),
        ShellPath::new("/usr/bin/zsh").unwrap(),
        &empty,
    );
    changes.want::<Dns>(Single, DnsProvider::Cloudflare, &empty);
    changes.want::<Hblock>(Single, Toggle::On, &empty);
    changes.want::<WifiBackends>(Single, WifiBackend::Iwd, &empty);
    changes.want::<GarudaUpdateFlags>(GarudaUpdateFlag::PacmanNoconfirm, Toggle::On, &empty);

    let (restored, problems) = restored(&changes, &empty);

    assert_eq!(problems, []);
    assert_eq!(restored.len(), 5);
    assert_eq!(restored.views(), changes.views());
}
