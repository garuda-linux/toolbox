use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::ChangeSet;
use toolbox_gui::bootrepair::BootRepairSetup;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup};
use toolbox_system::DryRunRepair;

use crate::common::{Fixture, TALL, window};
use crate::system::{settle, step_until};

pub fn app(forced: bool) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let setup = BootRepairSetup {
        runtime: runtime.handle().clone(),
        source: Arc::new(DryRunRepair),
        forced,
    };

    let mut harness = window(TALL).build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                boot_repair: Some(setup),
                ..Setup::default()
            },
        )
    });

    harness.run();
    Fixture::new(harness, runtime)
}

pub fn open(harness: &mut Harness<'static, App>) {
    harness
        .get_by_role_and_label(Role::Tab, "Boot Repair")
        .click();
    step_until(harness, "the disks", |harness| {
        harness.query_all_by_label("UEFI").next().is_some()
    });
    settle(harness);
}

#[test]
fn an_installed_system_does_not_list_the_page() {
    let mut fixture = app(false);
    let harness = fixture.harness();

    assert!(
        harness
            .query_by_role_and_label(Role::Tab, "Boot Repair")
            .is_none()
    );
}

#[test]
fn the_flag_shows_the_page_in_test_mode_and_a_repair_asks_first() {
    let mut fixture = app(true);
    let harness = fixture.harness();
    open(harness);

    assert!(
        harness
            .query_all_by_label_contains("Test mode")
            .next()
            .is_some()
    );

    harness
        .get_by_label("Reinstall the boot loader…")
        .scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall the boot loader…").click();
    harness.run();

    assert!(
        harness
            .query_all_by_label_contains("is written to the EFI system partition /dev/nvme0n1p2")
            .next()
            .is_some()
    );
}
