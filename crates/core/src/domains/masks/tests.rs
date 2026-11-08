use std::collections::BTreeSet;

use super::{MaskedUnits, MaskedUserUnits, SystemUnitMasks, UserUnitMasks};
use crate::domain::{Domain as _, Summary, Toggle};
use crate::domains::{EnabledUnits, EnabledUserUnits, SystemUnits, UserUnits};
use crate::testing::{restored, sided, unit};
use crate::{ChangeSet, Snapshot};

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(MaskedUnits(BTreeSet::from([unit("cups.service")])));
    snapshot.insert(MaskedUserUnits(BTreeSet::from([unit("psd.service")])));
    snapshot.insert(EnabledUnits::default());
    snapshot.insert(EnabledUserUnits::default());
    snapshot
}

#[test]
fn a_mask_matching_the_system_drops_out() {
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnitMasks>(unit("cups.service"), Toggle::On, &snapshot());
    changes.want::<UserUnitMasks>(unit("psd.service"), Toggle::On, &snapshot());

    assert_eq!(changes.views(), Vec::new());
}

#[test]
fn system_masks_go_through_the_helper_in_one_call_each() {
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnitMasks>(unit("bluetooth.service"), Toggle::On, &snapshot());
    changes.want::<SystemUnitMasks>(unit("avahi-daemon.service"), Toggle::On, &snapshot());
    changes.want::<SystemUnitMasks>(unit("cups.service"), Toggle::Off, &snapshot());

    assert_eq!(
        sided(&changes),
        [
            "root: systemctl mask -- avahi-daemon.service bluetooth.service",
            "root: systemctl unmask -- cups.service",
        ]
    );
}

#[test]
fn user_masks_run_as_the_user() {
    let mut changes = ChangeSet::default();
    changes.want::<UserUnitMasks>(unit("psd.service"), Toggle::Off, &snapshot());
    changes.want::<UserUnitMasks>(unit("tracker.service"), Toggle::On, &snapshot());

    assert_eq!(
        sided(&changes),
        [
            "user: systemctl --user mask -- tracker.service",
            "user: systemctl --user unmask -- psd.service",
        ]
    );
}

/// A masked unit cannot be enabled, so the unmask has to come first.
#[test]
fn an_unmask_runs_before_the_enable_of_the_same_unit() {
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnits>(unit("cups.service"), Toggle::On, &snapshot());
    changes.want::<SystemUnitMasks>(unit("cups.service"), Toggle::Off, &snapshot());
    changes.want::<UserUnits>(unit("psd.service"), Toggle::On, &snapshot());
    changes.want::<UserUnitMasks>(unit("psd.service"), Toggle::Off, &snapshot());

    assert_eq!(
        sided(&changes),
        [
            "root: systemctl unmask -- cups.service",
            "root: systemctl enable --now -- cups.service",
            "user: systemctl --user unmask -- psd.service",
            "user: systemctl --user enable --now -- psd.service",
        ]
    );
}

#[test]
fn masks_are_keyed_by_unit_and_name_it_in_their_sentence() {
    assert_eq!(SystemUnitMasks::key(&unit("cups.service")), "cups.service");
    assert_eq!(
        UserUnitMasks::summary(&unit("psd.service"), &Toggle::On),
        Summary::of("user-unit-mask", "psd.service")
    );
    assert_eq!(
        SystemUnitMasks::summary(&unit("cups.service"), &Toggle::Off),
        Summary::of("unit-unmask", "cups.service")
    );
    assert_eq!(SystemUnitMasks::ID, "unit-masks");
    assert_eq!(UserUnitMasks::ID, "user-unit-masks");
}

#[test]
fn queued_masks_survive_a_restart() {
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnitMasks>(unit("bluetooth.service"), Toggle::On, &snapshot());
    changes.want::<UserUnitMasks>(unit("tracker.service"), Toggle::On, &snapshot());

    let (restored, problems) = restored(&changes, &snapshot());

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
}
