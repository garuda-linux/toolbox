use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::packages::{InstallReason, LocalNode, PackageGraph};
use toolbox_core::{ChangeSet, Snapshot, Toggle};

use super::searching_firefox;
use crate::catalogs::Catalogs;
use crate::command::{AppCommand, Command, DiscoverCommand};
use crate::page::PageId;
use crate::testing::PageFixture;
use crate::toggle_pages::TogglePages;

fn catalogs() -> Catalogs {
    Catalogs::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog"))
}

fn node(name: &str, depends: &[&str], reason: InstallReason) -> LocalNode {
    LocalNode {
        name: name.parse().unwrap(),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        reason,
    }
}

/// Firefox alone pulled in `efibootmgr`, so the removal is refused.
fn firefox_takes_efibootmgr() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([
        "firefox".parse().unwrap(),
        "efibootmgr".parse().unwrap(),
    ])));
    snapshot.insert(SyncPackages(BTreeSet::from(["firefox".parse().unwrap()])));
    snapshot.insert(PackageGraph(vec![
        node("firefox", &["efibootmgr"], InstallReason::Explicit),
        node("efibootmgr", &[], InstallReason::Dependency),
    ]));
    snapshot
}

#[test]
fn a_refused_switch_shows_why_until_dismissed() {
    let snapshot = firefox_takes_efibootmgr();
    let mut pages = TogglePages::new(Arc::new(catalogs()));
    let handled = pages.handle(
        &AppCommand::Discover(DiscoverCommand::Toggle {
            app: "firefox".to_owned(),
            wish: Toggle::Off,
        }),
        &ChangeSet::default(),
        &snapshot,
    );

    assert!(handled.edit.is_none(), "the removal stays out of the queue");

    let mut fixture = PageFixture {
        catalogs: catalogs(),
        ..PageFixture::with_snapshot(snapshot)
    };

    fixture.services.toggle_pages = pages;

    let mut harness = fixture.harness(searching_firefox());
    harness.run();
    harness.get_by_label_contains("efibootmgr");

    harness
        .get_by_role_and_label(Role::Button, "Dismiss")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(AppCommand::DismissRefusal(PageId::Discover))]
    );
}
