use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_plan::{GrubMenu, GrubPlatform};
use toolbox_services::boot_menu::Menu;
use toolbox_system::MenuRead;

use super::super::tests::{grub_system, menu, page};
use crate::boot::Boot;

/// The `grub.cfg` of the Lima VM, another menu than the one of `menu()`.
const OTHER_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;

/// One spin field: the steppers touch the number, with no gap of a row.
#[test]
fn the_timeout_steppers_sit_inside_the_number_field() {
    let harness = page(grub_system(), Boot::showing(menu(), None, None));
    let rect = |role, label| harness.get_by_role_and_label(role, label).rect();
    let less = rect(Role::Button, "One second less");
    let number = rect(Role::TextInput, "Menu timeout in seconds");
    let more = rect(Role::Button, "One second more");

    assert!(
        (number.left() - less.right()).abs() < 0.5 && (more.left() - number.right()).abs() < 0.5,
        "{less:?} {number:?} {more:?}"
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
