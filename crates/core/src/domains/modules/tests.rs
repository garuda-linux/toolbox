use std::collections::BTreeSet;

use toolbox_plan::KernelModule;

use super::{ModuleBlacklist, OwnBlacklist};
use crate::domain::{Summary, Toggle};
use crate::test_support::{commands, restored, views};
use crate::{ChangeSet, Snapshot, plan};

fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

fn snapshot(own: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(OwnBlacklist(
        own.iter().map(|name| module(name)).collect::<BTreeSet<_>>(),
    ));
    snapshot
}

#[test]
fn a_wish_the_file_already_fulfils_drops_out() {
    let state = snapshot(&["wl"]);
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("wl"), Toggle::On, &state);
    changes.want::<ModuleBlacklist>(module("b43"), Toggle::Off, &state);

    assert!(changes.is_empty());
}

#[test]
fn the_dash_spelling_of_a_listed_module_counts_as_listed() {
    let state = snapshot(&["mt76_usb"]);
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("mt76-usb"), Toggle::On, &state);

    assert!(changes.is_empty());
}

#[test]
fn each_module_is_one_edit_of_the_apps_own_file() {
    let state = snapshot(&["wl"]);
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("r8169"), Toggle::On, &state);
    changes.want::<ModuleBlacklist>(module("wl"), Toggle::Off, &state);

    assert_eq!(
        commands(&plan(&changes)),
        [
            "add `blacklist r8169` in /etc/modprobe.d/garuda-toolbox-blacklist.conf",
            "remove `blacklist wl` in /etc/modprobe.d/garuda-toolbox-blacklist.conf",
        ]
    );

    assert_eq!(
        views(&changes),
        [
            (
                "module-blacklist",
                "r8169".to_owned(),
                Summary::of("driver-blacklist", "r8169")
            ),
            (
                "module-blacklist",
                "wl".to_owned(),
                Summary::of("driver-allow", "wl")
            ),
        ]
    );
}

#[test]
fn an_applied_change_leaves_the_queue_with_the_next_snapshot() {
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("r8169"), Toggle::On, &snapshot(&[]));
    assert_eq!(changes.len(), 1);

    changes.reconcile(&snapshot(&["r8169"]));
    assert!(changes.is_empty());
}

#[test]
fn queued_blacklist_changes_survive_a_save_and_restore() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();

    changes.want::<ModuleBlacklist>(module("r8169"), Toggle::On, &empty);
    changes.want::<ModuleBlacklist>(module("wl"), Toggle::Off, &empty);

    let (restored, problems) = restored(&changes, &empty);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
}
