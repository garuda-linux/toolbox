use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_plan::UnitName;

use super::{NetworkPage, facts, network};
use crate::testing::PageFixture;
use crate::units::{UnitListing, Units};

#[test]
fn a_service_state_is_a_dot_beside_the_name_as_on_services() {
    let manager = UnitInfo {
        name: UnitName::new("NetworkManager.service").unwrap(),
        description: "Network Manager".to_owned(),
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        file_state: FileState::Enabled,
    };

    let mut snapshot = Snapshot::default();
    snapshot.insert(facts());
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        network: network(),
        units: Units {
            system: UnitListing::Loaded(Arc::new(vec![manager])),
            ..Units::default()
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
