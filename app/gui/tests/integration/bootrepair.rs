use std::sync::Arc;

use egui::Theme;
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
use crate::look::look_for;
use crate::system::{settle, step_until};

pub fn app(forced: bool) -> Fixture {
    app_on(forced, Theme::Dark)
}

pub fn app_on(forced: bool, theme: Theme) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let look = look_for(&runtime, theme);
    let setup = BootRepairSetup {
        runtime: runtime.handle().clone(),
        source: Arc::new(DryRunRepair),
        forced,
    };

    let mut harness = window(TALL).with_theme(theme).build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                look,
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
        .get_by_label("Reinstall the Boot Loader…")
        .scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall the Boot Loader…").click();
    harness.run();

    assert!(
        harness
            .query_all_by_label_contains(
                "The boot loader on the EFI system partition /dev/nvme0n1p2"
            )
            .next()
            .is_some()
    );
}

#[test]
fn the_progress_banner_shows_while_the_repair_runs() {
    let mut fixture = app(true);
    let harness = fixture.harness();
    open(harness);

    harness.get_by_label("Reinstall the Boot Loader…").click();
    let field = "Passphrase of the encrypted partition";
    step_until(harness, "the passphrase field", |harness| {
        harness
            .query_by_role_and_label(Role::PasswordInput, field)
            .is_some()
    });
    harness
        .get_by_role_and_label(Role::PasswordInput, field)
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::PasswordInput, field)
        .type_text("not-a-real-passphrase");
    harness.run();
    harness
        .query_all_by_label("Reinstall the Boot Loader")
        .last()
        .unwrap()
        .click();
    step_until(harness, "the progress banner", |harness| {
        harness
            .query_by_label_contains("The repair is running")
            .is_some()
    });
    settle(harness);
    harness.snapshot("bootrepair/running_dark");
}
