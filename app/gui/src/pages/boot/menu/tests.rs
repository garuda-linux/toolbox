use toolbox_plan::{GrubMenu, GrubPlatform};
use toolbox_system::{Menu, MenuRead};

use super::super::tests::{grub_system, menu, page};
use crate::boot::Boot;

/// The `grub.cfg` of the Lima VM, another menu than the one of `menu()`.
const OTHER_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;

#[test]
fn the_entry_list_is_built_once_until_the_menu_changes() {
    let mut harness = page(grub_system(), Boot::showing(menu(), None, None));
    harness.run_steps(3);
    assert_eq!(harness.state().page.entries.builds(), 1);

    let other = Menu::Read(MenuRead::Known(GrubMenu::for_platform(
        OTHER_CFG,
        GrubPlatform::Efi,
    )));

    harness.state_mut().fixture.boot = Boot::showing(other, None, None);
    harness.run_steps(3);
    assert_eq!(harness.state().page.entries.builds(), 2);
}
