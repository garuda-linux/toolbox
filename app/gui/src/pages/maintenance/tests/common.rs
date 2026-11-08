use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::{page, programs, sent, with};
use crate::command::{Command, MaintenanceCommand};
use crate::maintenance::Maintenance;

#[test]
fn removing_orphans_asks_first_and_runs_on_its_danger_button() {
    let mut harness = page(with(programs(), Maintenance::default()));

    let orphans = harness
        .query_all_by_label("Run now")
        .nth(1)
        .expect("Remove orphans has a Run now button");

    orphans.click();
    harness.run();

    assert_eq!(sent(&harness), [], "nothing runs before the answer");
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Run Remove orphans?")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Remove orphans")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::RunNow(
            "remove-orphans".to_owned()
        ))]
    );
}

#[test]
fn queue_waits_in_the_menu_and_rests_while_a_plan_runs() {
    let mut harness = page(crate::testing::PageFixture {
        busy: true,
        ..with(programs(), Maintenance::default())
    });

    assert!(harness.query_by_label("Queue cleanup").is_none());
    harness
        .get_by_role_and_label(Role::Button, "More actions for Clean package cache")
        .click();
    harness.run();

    let queue = harness.get_by_role_and_label(Role::MenuItem, "Queue cleanup");
    assert!(queue.accesskit_node().is_disabled());
}
