use toolbox_plan::{PlannedAction, RootAction};

use super::{Hblock, HblockState};
use crate::domain::{Summary, Toggle};
use crate::test_support::actions;
use crate::{ChangeSet, Single, Snapshot, SummaryCode};

fn snapshot(on: bool) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(HblockState(on));
    snapshot
}

#[test]
fn the_current_state_drops_out() {
    let mut changes = ChangeSet::default();
    changes.want::<Hblock>(Single, Toggle::On, &snapshot(true));

    assert!(changes.is_empty());
}

#[test]
fn switching_on_installs_hblock_first() {
    let mut changes = ChangeSet::default();
    changes.want::<Hblock>(Single, Toggle::On, &snapshot(false));

    let actions = actions(&changes);

    assert!(matches!(
        actions.as_slice(),
        [
            PlannedAction::Root(RootAction::InstallPackages { packages }),
            PlannedAction::Root(RootAction::SetHblock { enabled: true }),
        ] if packages.iter().map(AsRef::as_ref).eq(["hblock"])
    ));
}

#[test]
fn switching_off_installs_nothing() {
    let mut changes = ChangeSet::default();
    changes.want::<Hblock>(Single, Toggle::Off, &snapshot(true));

    assert_eq!(
        actions(&changes),
        [PlannedAction::Root(RootAction::SetHblock {
            enabled: false
        })]
    );
    assert_eq!(
        changes
            .views()
            .first()
            .map(|view| (view.key.clone(), view.summary.clone())),
        Some(("hblock".to_owned(), Summary::plain(SummaryCode::HblockOff)))
    );
}
