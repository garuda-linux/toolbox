use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, UserName, WifiBackend};
use toolbox_state::probes::{FailedProbes, Probe};

use super::{Choices, Flip, Pick, Read};
use crate::domains::{
    AvailableShells, CurrentDns, CurrentShells, CurrentWifiBackend, Dns, GarudaUpdateConfig,
    GarudaUpdateFlags, Hblock, HblockState, LoginShells,
};
use crate::{ChangeSet, Single, Snapshot, Toggle};

fn chosen<T>(pick: &Pick<T>) -> Option<&T> {
    pick.options.get(pick.shown?)
}

fn user() -> UserName {
    UserName::new("tester").unwrap()
}

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

#[test]
fn nothing_probed_is_unknown_and_shows_no_option() {
    let choices = Choices::of(&Snapshot::default(), &ChangeSet::default(), Some(&user()));

    for read in [choices.shell.read, choices.dns.read, choices.wifi.read] {
        assert_eq!(read, Read::NotYet);
    }

    assert!(!choices.hblock.known);
    assert_eq!(choices.shell.options, []);
    assert_eq!(choices.dns.shown, None);
    assert_eq!(choices.dns.options.len(), DnsProvider::ALL.len());
    assert_eq!(choices.flags, None);
}

#[test]
fn the_shell_shows_the_queued_choice_among_the_systems_shells() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentShells(BTreeMap::from([(
        user(),
        shell("/usr/bin/bash"),
    )])));
    snapshot.insert(AvailableShells(vec![
        shell("/usr/bin/bash"),
        shell("/usr/bin/fish"),
    ]));
    let mut changes = ChangeSet::default();

    let system = Choices::of(&snapshot, &changes, Some(&user())).shell;
    assert_eq!(
        (system.read, system.shown, system.queued),
        (Read::Known, Some(0), false)
    );

    changes.want::<LoginShells>(user(), shell("/usr/bin/fish"), &snapshot);
    let queued = Choices::of(&snapshot, &changes, Some(&user())).shell;
    assert_eq!((queued.shown, queued.queued), (Some(1), true));
}

#[test]
fn a_shell_the_system_does_not_list_is_none_of_the_options() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentShells(BTreeMap::from([(
        user(),
        shell("/usr/bin/nologin"),
    )])));
    snapshot.insert(AvailableShells(vec![shell("/usr/bin/bash")]));

    let choices = Choices::of(&snapshot, &ChangeSet::default(), Some(&user()));

    assert_eq!(choices.shell.read, Read::Known);
    assert_eq!(choices.shell.shown, None);
}

#[test]
fn dns_and_wifi_show_the_queued_wish_before_the_systems() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentDns(DnsProvider::Default));
    snapshot.insert(CurrentWifiBackend(WifiBackend::Iwd));
    let mut changes = ChangeSet::default();

    let system = Choices::of(&snapshot, &changes, None);
    assert_eq!(chosen(&system.dns), Some(&DnsProvider::Default));
    assert_eq!(chosen(&system.wifi), Some(&WifiBackend::Iwd));

    changes.want::<Dns>(Single, DnsProvider::Quad9, &snapshot);
    let queued = Choices::of(&snapshot, &changes, None);
    assert_eq!(chosen(&queued.dns), Some(&DnsProvider::Quad9));
    assert!(queued.dns.queued);
}

#[test]
fn switches_show_the_queued_wish_before_the_systems_state() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(HblockState(false));
    snapshot.insert(GarudaUpdateConfig(BTreeSet::from([
        GarudaUpdateFlag::UpdateAur,
    ])));
    let mut changes = ChangeSet::default();
    changes.want::<Hblock>(Single, Toggle::On, &snapshot);
    changes.want::<GarudaUpdateFlags>(GarudaUpdateFlag::UpdateAur, Toggle::Off, &snapshot);

    let choices = Choices::of(&snapshot, &changes, None);

    assert_eq!(
        choices.hblock,
        Flip {
            known: true,
            on: true,
            queued: true,
        }
    );

    let flags = choices.flags.unwrap();
    assert_eq!(flags.len(), GarudaUpdateFlag::ALL.len());
    assert!(flags.contains(&(
        GarudaUpdateFlag::UpdateAur,
        Flip {
            known: true,
            on: false,
            queued: true,
        }
    )));
}

/// A probed snapshot names the probes that failed. A part missing
/// without its probe there is one the system lacks.
#[test]
fn a_missing_part_of_a_probed_system_is_failed_or_absent_by_its_probe() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(FailedProbes::new([Probe::Dns, Probe::AvailableShells]));

    let choices = Choices::of(&snapshot, &ChangeSet::default(), Some(&user()));

    assert_eq!(
        [choices.shell.read, choices.dns.read, choices.wifi.read],
        [Read::Failed, Read::Failed, Read::Absent]
    );
}

#[test]
fn a_part_that_was_read_is_known_whatever_failed_besides() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(FailedProbes::new([Probe::Dns]));
    snapshot.insert(CurrentWifiBackend(WifiBackend::Iwd));

    let choices = Choices::of(&snapshot, &ChangeSet::default(), None);

    assert_eq!(choices.wifi.read, Read::Known);
}
