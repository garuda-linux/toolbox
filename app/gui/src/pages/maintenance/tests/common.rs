use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::maintenance::{self, Operation, Task};
use toolbox_plan::Toggle;

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
        [Command::from(MaintenanceCommand::RunOperation(
            Operation::RemoveOrphans
        ))]
    );
}

#[test]
fn a_queued_operation_marks_its_row() {
    let mut fixture = with(programs(), Maintenance::default());
    let snapshot = std::sync::Arc::clone(&fixture.snapshot);
    maintenance::queue(
        Task::CleanPackageCache,
        Toggle::On,
        &mut fixture.changes,
        &snapshot,
    );

    let harness = page(fixture);

    assert_eq!(
        harness.query_all_by_label("Runs with Apply").count(),
        1,
        "only the queued row carries the mark"
    );
}

#[test]
fn queue_sits_beside_run_now_and_rests_while_a_plan_runs() {
    let harness = page(crate::testing::PageFixture {
        busy: true,
        ..with(programs(), Maintenance::default())
    });

    let queue = harness.get_by_role_and_label(Role::Button, "Queue cleanup");
    assert!(queue.accesskit_node().is_disabled());
    assert!(harness.query_by_role(Role::MenuItem).is_none());
}

#[test]
fn a_queued_operation_offers_to_take_it_back() {
    let mut fixture = with(programs(), Maintenance::default());
    let snapshot = std::sync::Arc::clone(&fixture.snapshot);
    maintenance::queue(
        Task::CleanPackageCache,
        Toggle::On,
        &mut fixture.changes,
        &snapshot,
    );

    let mut harness = page(fixture);
    harness.get_by_label("Remove from queue").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::Queue {
            operation: Operation::CleanPackageCache,
            on: false,
        })]
    );
}
