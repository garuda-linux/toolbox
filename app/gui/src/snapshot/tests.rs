use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_system::{Probe, ProbeError, ProbeIssue};

use super::{FailedRead, Probed, SnapshotHandle, spawn, warn_on_change};
use crate::testing::PATIENCE;

fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(
        names
            .iter()
            .map(|name| name.parse().unwrap())
            .collect::<BTreeSet<_>>(),
    ));
    snapshot
}

async fn first_read(mut handle: SnapshotHandle) -> Arc<Probed> {
    tokio::time::timeout(PATIENCE, async {
        loop {
            if let Some(probed) = handle
                .changed()
                .and_then(|reading| reading.latest().cloned())
            {
                return probed;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn the_probe_fills_the_snapshot() {
    let handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        || (installed(&["firefox"]), Vec::new()),
    );

    let probed = first_read(handle).await;

    assert_eq!(probed.failed, Vec::new());
    assert_eq!(
        probed.snapshot.part::<InstalledPackages>(),
        installed(&["firefox"]).part::<InstalledPackages>()
    );
}

#[tokio::test]
async fn only_failed_probes_are_kept() {
    let handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        || {
            let issues = vec![
                ProbeIssue {
                    probe: Probe::Grub,
                    error: ProbeError::Unavailable,
                },
                ProbeIssue {
                    probe: Probe::Units,
                    error: ProbeError::Failed("no bus".to_owned()),
                },
            ];

            (Snapshot::default(), issues)
        },
    );

    assert_eq!(
        first_read(handle).await.failed,
        vec![FailedRead {
            probe: Probe::Units,
            reason: "no bus".to_owned(),
        }]
    );
}

/// Every read repeats a lasting failure, the log names it once.
#[test]
fn a_failure_is_logged_once_until_it_changes() {
    let logged = Mutex::new(Vec::new());
    let units = FailedRead {
        probe: Probe::Units,
        reason: "no bus".to_owned(),
    };

    let grub = FailedRead {
        probe: Probe::Grub,
        reason: "/etc/default/grub: Permission denied".to_owned(),
    };

    assert!(warn_on_change(&logged, std::slice::from_ref(&units)));
    assert!(!warn_on_change(&logged, std::slice::from_ref(&units)));
    assert!(warn_on_change(&logged, &[units, grub]));
    assert!(warn_on_change(&logged, &[]));
    assert!(!warn_on_change(&logged, &[]));
}
