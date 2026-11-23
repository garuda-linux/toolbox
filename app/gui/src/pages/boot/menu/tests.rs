use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::CurrentGrub;
use toolbox_core::features::boot::edit::Edit;
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState};
use toolbox_services::boot_menu::Menu;
use toolbox_system::MenuRead;

use super::super::tests::{Shown, grub_system, menu, page};
use crate::boot::Boot;
use crate::command::{BootCommand, Command};

/// The `grub.cfg` of the Lima VM, another menu than the one of `menu()`.
const OTHER_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;

/// One spin field: the steppers touch the number, with no gap of a row.
#[test]
fn the_timeout_steppers_sit_inside_the_number_field() {
    let harness = page(grub_system(), Boot::showing(menu(), None, None));
    let rect = |role, label| harness.get_by_role_and_label(role, label).rect();
    let less = rect(Role::Button, "One second less");
    let number = rect(Role::SpinButton, "Menu timeout in seconds");
    let more = rect(Role::Button, "One second more");

    // Breeze stacks the arrows in one column at the field's right edge, each
    // within the field's height.
    assert!(
        (less.left() - number.right()).abs() < 0.5 && (more.left() - number.right()).abs() < 0.5,
        "the arrows leave the number: {less:?} {number:?} {more:?}"
    );
    assert!(
        (more.bottom() - less.top()).abs() < 0.5 || (less.bottom() - more.top()).abs() < 0.5,
        "the arrows do not stack: {less:?} {more:?}"
    );
    assert!(
        less.top() >= number.top()
            && more.top() >= number.top()
            && less.bottom() <= number.bottom()
            && more.bottom() <= number.bottom(),
        "the arrows leave the field's height: {less:?} {more:?} {number:?}"
    );
}

#[test]
fn the_entry_list_is_built_once_until_the_menu_changes() {
    let mut harness = page(grub_system(), Boot::showing(menu(), None, None));
    harness.run_steps(3);
    assert_eq!(harness.state().page.entries.builds(), 1);

    let other = Menu::Read(MenuRead::Known(GrubMenu::for_platform(
        OTHER_CFG,
        GrubPlatform::Efi,
    )));

    harness.state_mut().fixture.services.boot = Boot::showing(other, None, None);
    harness.run_steps(3);
    assert_eq!(harness.state().page.entries.builds(), 2);
}

/// The install's file with its timeout changed to GRUB's -1.
fn waiting() -> Snapshot {
    let defaults = toolbox_plan::testing::grub::INSTALLED_DEFAULT_GRUB;
    let file = defaults.replace("GRUB_TIMEOUT=5", "GRUB_TIMEOUT=-1");
    let mut snapshot = grub_system();

    assert_ne!(file, defaults);
    snapshot.insert(CurrentGrub(GrubState::read(&file)));
    snapshot
}

#[test]
fn a_timeout_of_minus_one_shows_as_waiting_for_a_key_without_seconds() {
    let harness = page(waiting(), Boot::showing(menu(), None, None));

    let mode = harness.get_by_role_and_label(Role::ComboBox, "Menu timeout");
    assert_eq!(mode.value().as_deref(), Some("The menu waits for a key"));
    assert!(
        harness
            .query_by_role_and_label(Role::SpinButton, "Menu timeout in seconds")
            .is_none()
    );
}

#[test]
fn the_timeout_mode_queues_minus_one_or_seconds() {
    let mut harness = page(grub_system(), Boot::showing(menu(), None, None));
    pick(&mut harness, "The menu waits for a key");
    assert_eq!(harness.state().fixture.sent(), [timeout(-1)]);

    let mut again = page(waiting(), Boot::showing(menu(), None, None));
    pick(&mut again, "Seconds");
    assert_eq!(again.state().fixture.sent(), [timeout(5)]);
}

fn pick(harness: &mut Shown, choice: &str) {
    harness
        .get_by_role_and_label(Role::ComboBox, "Menu timeout")
        .click();
    harness.run();
    harness.get_by_label(choice).click();
    harness.run();
}

fn timeout(seconds: i64) -> Command {
    Command::from(BootCommand::Edit(Edit::Timeout(seconds)))
}
