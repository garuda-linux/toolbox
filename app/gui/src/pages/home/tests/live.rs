//! Home on the live ISO.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_system::Host;

use super::page;
use crate::programs::Programs;
use crate::testing::PageFixture;

fn live(programs: Programs) -> PageFixture {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Host {
        user: None,
        live_iso: true,
        desktops: Vec::new(),
        edition: None,
        reboot_pending: false,
    });

    PageFixture {
        programs: Some(programs),
        ..PageFixture::with_snapshot(snapshot)
    }
}

#[test]
fn without_installer_and_repair_shell_there_is_no_live_heading() {
    let harness = page(live(Programs::default()));

    assert!(harness.query_by_label("Live System").is_none());
    assert!(harness.query_by_label("Start Installer").is_none());
}

#[test]
fn the_installer_comes_under_the_live_heading() {
    let harness = page(live(Programs {
        installer: true,
        ..Programs::default()
    }));

    assert!(harness.query_by_label("Live System").is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Start Installer")
            .is_some()
    );
}
