use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_plan::UnitName;

use super::{NetworkPage, facts, network};
use crate::page_service::Services;
use crate::testing::PageFixture;
use crate::units::{UnitListing, UnitLists, Units};

fn network_manager() -> UnitInfo {
    UnitInfo {
        name: UnitName::new("NetworkManager.service").unwrap(),
        description: "Network Manager".to_owned(),
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        file_state: FileState::Enabled,
    }
}

#[test]
fn a_service_state_is_a_dot_beside_the_name_as_on_services() {
    let manager = network_manager();

    let mut snapshot = Snapshot::default();
    snapshot.insert(facts());
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        services: Services {
            network: network(),
            units: Units::showing(UnitLists {
                system: UnitListing::Loaded(Arc::new(vec![manager])),
                ..UnitLists::default()
            }),
            ..Services::default()
        },
        ..PageFixture::default()
    }
    .harness(NetworkPage::default());

    harness.run();
    let name = harness
        .get_by_role_and_label(Role::Label, "NetworkManager")
        .rect();

    let state = harness.get_by_label("Running").rect();

    assert!(
        state.left() > name.right() && (state.center().y - name.center().y).abs() < 2.0,
        "Running at {state:?} is not beside the name at {name:?}"
    );
    let dot = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, egui::Shape::Circle(circle)
            if state.y_range().contains(circle.center.y) && circle.center.x <= state.left() + 8.0)
    });

    assert!(dot, "no dot before Running");
}

#[test]
fn a_service_that_is_not_installed_is_not_listed() {
    let manager = network_manager();

    let mut harness = PageFixture {
        services: Services {
            network: network(),
            units: Units::showing(UnitLists {
                system: UnitListing::Loaded(Arc::new(vec![manager])),
                ..UnitLists::default()
            }),
            ..Services::default()
        },
        ..PageFixture::default()
    }
    .harness(NetworkPage::default());

    harness.run();

    assert!(harness.query_by_label("ConnMan").is_none());
    assert!(harness.query_by_label("Not installed").is_none());
}
