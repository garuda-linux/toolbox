use std::collections::{BTreeMap, BTreeSet};

use toolbox_core::domains::{
    AvailableShells, CurrentDns, CurrentShells, CurrentWifiBackend, Dns, GarudaUpdateConfig,
    GarudaUpdateFlags, Hblock, HblockState, LoginShells,
};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Single, Snapshot, Toggle};
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, UserName, WifiBackend};

use super::{Choices, Flip, Pick};

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

    assert!(!choices.shell.known && !choices.dns.known && !choices.wifi.known);
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
        (system.known, system.shown, system.queued),
        (true, Some(0), false)
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

    assert!(choices.shell.known);
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
