use toolbox_plan::{DnsProvider, PlannedAction, WifiBackend};

use super::{CurrentDns, CurrentWifiBackend, Dns, WifiBackends};
use crate::domain::Summary;
use crate::{ChangeSet, Single, Snapshot, plan};

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentDns(DnsProvider::Default));
    snapshot.insert(CurrentWifiBackend(WifiBackend::WpaSupplicant));
    snapshot
}

fn shown(changes: &ChangeSet) -> Vec<String> {
    plan(changes)
        .steps()
        .iter()
        .map(|step| match &step.action {
            PlannedAction::Root(action) => format!("{action:?}"),
            PlannedAction::User(action) => format!("user {action:?}"),
        })
        .collect()
}

#[test]
fn the_current_choices_drop_out() {
    let mut changes = ChangeSet::default();
    changes.want::<Dns>(Single, DnsProvider::Default, &snapshot());
    changes.want::<WifiBackends>(Single, WifiBackend::WpaSupplicant, &snapshot());

    assert!(changes.is_empty());
}

#[test]
fn a_dns_provider_is_one_root_action() {
    let mut changes = ChangeSet::default();
    changes.want::<Dns>(Single, DnsProvider::Quad9, &snapshot());

    assert_eq!(shown(&changes), ["SetDns { provider: Quad9 }"]);
    assert_eq!(
        changes.views().first().map(|view| view.summary.clone()),
        Some(Summary::of("dns", "quad9"))
    );
}

#[test]
fn switching_to_iwd_installs_it_first() {
    let mut changes = ChangeSet::default();
    changes.want::<WifiBackends>(Single, WifiBackend::Iwd, &snapshot());

    assert_eq!(
        shown(&changes),
        [
            "InstallPackages { packages: TargetList { targets: [\"iwd\"] } }",
            "SetWifiBackend { backend: Iwd }"
        ]
    );
}

#[test]
fn switching_back_needs_no_package() {
    let mut snapshot = snapshot();
    snapshot.insert(CurrentWifiBackend(WifiBackend::Iwd));
    let mut changes = ChangeSet::default();
    changes.want::<WifiBackends>(Single, WifiBackend::WpaSupplicant, &snapshot);

    assert_eq!(
        shown(&changes),
        ["SetWifiBackend { backend: WpaSupplicant }"]
    );
    assert_eq!(
        changes.views().first().map(|view| view.key.clone()),
        Some("wifi-backend".to_owned())
    );
}

#[test]
fn the_wifi_switch_runs_after_dns_and_the_services() {
    let mut changes = ChangeSet::default();
    changes.want::<WifiBackends>(Single, WifiBackend::Iwd, &snapshot());
    changes.want::<Dns>(Single, DnsProvider::Google, &snapshot());

    let shown = shown(&changes);

    assert_eq!(
        shown.first().map(String::as_str),
        Some("SetDns { provider: Google }")
    );
    assert_eq!(
        shown.last().map(String::as_str),
        Some("SetWifiBackend { backend: Iwd }")
    );
}
