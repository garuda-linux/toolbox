use std::sync::Arc;
use std::time::Duration;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use futures_lite::future::Boxed;
use tokio::runtime::Runtime;
use toolbox_core::ChangeSet;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_gui::queue::Queue;
use toolbox_gui::units::{self, UnitList, UnitLister};
use toolbox_gui::{App, Setup};
use toolbox_plan::{UnitName, UnitScope};
use toolbox_system::CallError;

use crate::common::{HEIGHT, no_note, type_into, window};
use crate::system::step_until;

/// Three system units, and no session bus for the user's.
struct Fake;

impl UnitLister for Fake {
    fn list(&self, scope: UnitScope) -> Boxed<UnitList> {
        Box::pin(async move {
            match scope {
                UnitScope::System => Ok(Arc::new(vec![
                    unit(
                        "sshd.service",
                        ActiveState::Active,
                        "running",
                        FileState::Enabled,
                    ),
                    unit(
                        "bluetooth.service",
                        ActiveState::Failed,
                        "failed",
                        FileState::Enabled,
                    ),
                    unit(
                        "cups.service",
                        ActiveState::Inactive,
                        "dead",
                        FileState::Disabled,
                    ),
                ])),
                UnitScope::User => Err(CallError::Unavailable),
            }
        })
    }
}

fn unit(name: &str, active: ActiveState, sub: &str, file_state: FileState) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: format!("About {name}"),
        active,
        sub_state: sub.to_owned(),
        file_state,
    }
}

pub fn services() -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let units = units::spawn(
            &handle,
            toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
            Arc::new(Fake),
            Duration::ZERO,
        );

        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                units: Some(units),
                ..Setup::default()
            },
        )
    });

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num7);
    step_until(&mut harness, "the units", |harness| {
        listed(harness, "sshd.service")
    });
    harness.run();
    (harness, runtime)
}

/// Whether `unit` has a row, found by its switch: its name shows twice.
fn listed(harness: &Harness<'static, App>, unit: &str) -> bool {
    harness
        .query_by_role_and_label(Role::CheckBox, &format!("Start {unit} at boot"))
        .is_some()
}

pub fn open_menu(harness: &mut Harness<'static, App>, unit: &str) {
    harness.get_by_label(&format!("Actions for {unit}")).click();
    harness.run();
}

#[test]
fn running_and_failed_units_show_and_the_chips_add_the_others() {
    let (mut harness, _runtime) = services();

    assert!(
        harness.query_by_label("Running (running)").is_none(),
        "the sub state repeats"
    );
    assert!(harness.query_by_label("Running").is_some());
    assert!(harness.query_by_label("Failed").is_some());
    assert!(!listed(&harness, "cups.service"));

    harness.get_by_label("Hide disabled").click();
    harness.get_by_label("Hide inactive").click();
    harness.run();
    assert!(listed(&harness, "cups.service"));
}

#[test]
fn the_switch_queues_the_boot_state() {
    let (mut harness, _runtime) = services();

    harness
        .get_by_role_and_label(Role::CheckBox, "Start sshd.service at boot")
        .click();
    harness.run();

    assert!(harness.query_by_label("Disables with Apply").is_some());
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn the_menu_restarts_at_once_and_queues_a_mask() {
    let (mut harness, _runtime) = services();

    open_menu(&mut harness, "sshd.service");
    harness
        .get_by_role_and_label(Role::MenuItem, "Restart")
        .click();
    harness.run();
    assert!(harness.query_by_label("Hide the Console").is_some());

    harness.get_by_label("Hide the Console").click();
    harness.run();

    open_menu(&mut harness, "sshd.service");
    harness
        .get_by_role_and_label(Role::MenuItem, "Mask")
        .click();
    harness.run();
    assert!(harness.query_by_label("Masks with Apply").is_some());
}

#[test]
fn the_user_scope_says_why_it_has_no_units() {
    let (mut harness, _runtime) = services();

    // The user units failed while Home was open. Only their page tells.
    assert!(no_note(&harness));

    harness
        .get_by_role_and_label(Role::RadioButton, "User")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_label("systemd is not reachable.")
            .is_some()
    );
}

#[test]
fn the_search_keeps_matching_units() {
    let (mut harness, _runtime) = services();

    type_into(&mut harness, "Search services", "blue");

    assert!(listed(&harness, "bluetooth.service"));
    assert!(!listed(&harness, "sshd.service"));
}
