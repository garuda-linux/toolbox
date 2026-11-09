//! Stopping a unit the session lives on asks first.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_plan::{UnitName, UnitOperation, UnitScope};

use super::{ServicesPage, loaded, unit};
use crate::command::{Command, ServiceCommand};

fn network_manager() -> UnitInfo {
    UnitInfo {
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        ..unit(
            "NetworkManager.service",
            "Network Manager",
            FileState::Enabled,
        )
    }
}

#[test]
fn stopping_the_network_asks_first_and_stops_on_the_danger_button() {
    let fixture = loaded(vec![network_manager()]);
    let mut harness = fixture.harness(ServicesPage::default());
    harness.run();

    harness
        .get_by_role_and_label(Role::Button, "Actions for NetworkManager.service")
        .click();
    harness.run();
    harness.get_by_label("Stop").click();
    harness.run();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [],
        "nothing stops before the answer"
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Stop NetworkManager.service?")
            .is_some()
    );

    harness.get_by_role_and_label(Role::Button, "Stop").click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(ServiceCommand::Control {
            scope: UnitScope::System,
            unit: UnitName::new("NetworkManager.service").unwrap(),
            operation: UnitOperation::Stop,
        })]
    );
}

/// Leaving the page drops the question, so returning shows no confirm
/// for a stop the user walked away from.
#[test]
fn leaving_the_page_drops_the_open_question() {
    use crate::pages::PageView as _;

    let fixture = loaded(vec![network_manager()]);
    let mut harness = fixture.harness(ServicesPage::default());
    harness.run();

    harness
        .get_by_role_and_label(Role::Button, "Actions for NetworkManager.service")
        .click();
    harness.run();
    harness.get_by_label("Stop").click();
    harness.run();
    harness.run();

    let ctx = harness.ctx.clone();

    harness.state_mut().page.left(&ctx);
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Stop NetworkManager.service?")
            .is_none()
    );
    assert_eq!(harness.state().fixture.sent(), []);
}
