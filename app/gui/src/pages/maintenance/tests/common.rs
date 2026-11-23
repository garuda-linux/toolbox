use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::maintenance::page::LaunchCause;
use toolbox_core::features::maintenance::{self, Operation, Task};
use toolbox_plan::Toggle;

use super::{open_tab, page, pending, programs, sent, with};
use crate::command::{Command, MaintenanceCommand};
use crate::maintenance::{Maintenance, MaintenanceProblem};
use crate::pages::PageView as _;
use crate::pages::maintenance::{MaintenancePage, Tab};
use crate::testing::PageFixture;

#[test]
fn removing_orphans_asks_first_and_runs_on_its_danger_button() {
    let mut harness = page(with(programs(), Maintenance::default()));

    let orphans = harness
        .query_all_by_label("Run Now")
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
    let harness = page(PageFixture {
        busy: true,
        ..with(programs(), Maintenance::default())
    });

    let queue = harness.get_by_role_and_label(Role::Button, "Queue Cleanup");
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
    harness.get_by_label("Remove from Queue").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::Queue {
            operation: Operation::CleanPackageCache,
            on: false,
        })]
    );
}

#[test]
fn before_the_programs_are_found_placeholder_rows_wait() {
    for (tab, title) in [("Common", "Tools"), ("Repairs", "Repairs")] {
        let mut harness = pending(PageFixture::default());
        open_tab(&mut harness, tab);

        let waiting = harness.get_by_role_and_label(Role::ProgressIndicator, title);
        assert!(waiting.rect().height() > 100.0, "{tab}: rows, not a line");
    }
}

/// A question belongs to its tab, and to the page: another tab or page
/// closes it unanswered.
#[test]
fn a_question_closes_when_its_tab_or_page_is_left() {
    let leaves: [fn(&mut MaintenancePage, &egui::Context); 2] = [
        |shown, _| shown.show_tab(Tab::Repairs),
        |shown, ctx| shown.left(ctx),
    ];

    for leave in leaves {
        let mut harness = page(with(programs(), Maintenance::default()));
        harness
            .query_all_by_label("Run Now")
            .nth(1)
            .unwrap()
            .click();
        harness.run();
        assert!(harness.query_by_role(Role::Dialog).is_some());

        let ctx = harness.ctx.clone();
        leave(&mut harness.state_mut().page, &ctx);
        harness.run();

        assert!(harness.query_by_role(Role::Dialog).is_none());
        assert_eq!(sent(&harness), []);
    }
}

#[test]
fn a_tool_missing_after_its_install_names_the_program() {
    let problem = MaintenanceProblem::Launch {
        tool: "btrfs-assistant",
        cause: LaunchCause::AbsentAfterInstall,
        reason: "/usr/bin/btrfs-assistant".to_owned(),
    };

    let maintenance = Maintenance::showing(Some(problem), false, Vec::new(), None);
    let harness = page(with(programs(), maintenance));

    assert!(
        harness
            .query_by_label_contains("is not there, so Btrfs Assistant did not start.")
            .is_some()
    );
}
