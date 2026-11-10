use std::collections::BTreeSet;

use toolbox_plan::{DnsProvider, GarudaUpdateFlag, WifiBackend};

use super::{Tweak, queue};
use crate::domains::{
    CurrentDns, CurrentWifiBackend, Dns, GarudaUpdateConfig, GarudaUpdateFlags, Hblock,
    HblockState, WifiBackends,
};
use crate::{ChangeSet, Single, Snapshot, Toggle};

/// Google's servers, `wpa_supplicant`, hblock off and garuda-update's AUR
/// updates on.
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

fn queued(tweak: Tweak) -> ChangeSet {
    let mut changes = ChangeSet::default();
    queue(&mut changes, tweak, &system());
    changes
}

#[test]
fn each_tweak_queues_its_domains_wish() {
    assert_eq!(
        queued(Tweak::Dns(DnsProvider::Quad9)).wish::<Dns>(&Single),
        Some(&DnsProvider::Quad9)
    );
    assert_eq!(
        queued(Tweak::Wifi(WifiBackend::Iwd)).wish::<WifiBackends>(&Single),
        Some(&WifiBackend::Iwd)
    );
    assert_eq!(
        queued(Tweak::Hblock(Toggle::On)).wish::<Hblock>(&Single),
        Some(&Toggle::On)
    );
    assert_eq!(
        queued(Tweak::Flag(GarudaUpdateFlag::NoCheckSpace, Toggle::On))
            .wish::<GarudaUpdateFlags>(&GarudaUpdateFlag::NoCheckSpace),
        Some(&Toggle::On)
    );
}

#[test]
fn a_tweak_the_system_has_is_not_queued() {
    for tweak in [
        Tweak::Dns(DnsProvider::Google),
        Tweak::Wifi(WifiBackend::WpaSupplicant),
        Tweak::Hblock(Toggle::Off),
        Tweak::Flag(GarudaUpdateFlag::UpdateAur, Toggle::On),
    ] {
        assert!(queued(tweak.clone()).is_empty(), "{tweak:?}");
    }
}
