use std::collections::BTreeSet;

use clap::Parser as _;
use toolbox_cli::args::{Cli, Command, Group, TweaksCommand};
use toolbox_core::domains::{
    CurrentDns, CurrentWifiBackend, Dns, GarudaUpdateConfig, GarudaUpdateFlags, Hblock,
    HblockState, WifiBackends,
};
use toolbox_core::{ChangeSet, Single, Snapshot, Toggle};
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, WifiBackend};

use super::{onto, rows};

fn tweak(words: &[&str]) -> TweaksCommand {
    let argv = ["garuda-toolbox", "tweaks"]
        .into_iter()
        .chain(words.iter().copied());

    match Cli::try_parse_from(argv).unwrap().command {
        Some(Command::Tweaks(Group(command))) => command,
        other => panic!("not a tweak: {other:?}"),
    }
}

/// Google's servers, `wpa_supplicant`, hblock off and garuda-update's
/// AUR updates on.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(CurrentDns(DnsProvider::Google));
    snapshot.insert(CurrentWifiBackend(WifiBackend::WpaSupplicant));
    snapshot.insert(HblockState(false));
    snapshot.insert(GarudaUpdateConfig(BTreeSet::from([
        GarudaUpdateFlag::UpdateAur,
    ])));
    snapshot
}

fn queued(words: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    onto(&tweak(words), &system(), &mut changes);
    changes
}

#[test]
fn each_tweak_queues_the_wish_the_page_queues() {
    assert_eq!(
        queued(&["dns", "cloudflare"]).wish::<Dns>(&Single),
        Some(&DnsProvider::Cloudflare)
    );
    assert_eq!(
        queued(&["wifi-backend", "iwd"]).wish::<WifiBackends>(&Single),
        Some(&WifiBackend::Iwd)
    );
    assert_eq!(
        queued(&["hblock", "on"]).wish::<Hblock>(&Single),
        Some(&Toggle::On)
    );
    assert_eq!(
        queued(&["update-flag", "skip-mirrorlist", "on"])
            .wish::<GarudaUpdateFlags>(&GarudaUpdateFlag::SkipMirrorlist),
        Some(&Toggle::On)
    );
}

#[test]
fn what_the_system_has_already_queues_nothing() {
    for words in [
        ["dns", "google"].as_slice(),
        &["wifi-backend", "wpa-supplicant"],
        &["hblock", "off"],
        &["update-flag", "update-aur", "on"],
    ] {
        assert!(queued(words).is_empty(), "{words:?}");
    }
}

#[test]
fn a_name_the_page_lacks_is_no_command() {
    let argv = ["garuda-toolbox", "tweaks", "dns", "opendns"];

    Cli::try_parse_from(argv).unwrap_err();
}

#[test]
fn the_list_shows_each_tweak_as_it_is_now() {
    let shown: Vec<(String, String)> = rows(&system())
        .into_iter()
        .map(|row| (row.id, row.value))
        .collect();

    assert_eq!(
        shown,
        [
            ("dns", "google"),
            ("wifi-backend", "wpa-supplicant"),
            ("hblock", "off"),
            ("update-flag skip-mirrorlist", "off"),
            ("update-flag update-aur", "on"),
            ("update-flag pacman-noconfirm", "off"),
            ("update-flag no-check-space", "off"),
        ]
        .map(|(id, value)| (id.to_owned(), value.to_owned()))
    );
}

#[test]
fn a_part_not_read_yet_says_so() {
    let shown = rows(&Snapshot::default());

    assert!(shown.iter().all(|row| row.value == "not read"));
}
