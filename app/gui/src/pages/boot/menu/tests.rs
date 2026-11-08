use toolbox_plan::GrubMenu;
use toolbox_system::{Menu, MenuRead};

use super::super::tests::{grub_system, menu, page};
use crate::boot::Boot;

const OTHER_CFG: &str = "\
menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {
}
menuentry 'Windows' $menuentry_id_option 'osprober-chain-1' {
}
";

#[test]
fn the_entry_list_is_built_once_until_the_menu_changes() {
    let mut harness = page(grub_system(), Boot::showing(menu(), None, None));
    harness.run_steps(3);
    assert_eq!(harness.state().page.entries.builds(), 1);

    let other = Menu::Read(MenuRead::Known(GrubMenu::from_config(OTHER_CFG)));
    harness.state_mut().fixture.boot = Boot::showing(other, None, None);
    harness.run_steps(3);
    assert_eq!(harness.state().page.entries.builds(), 2);
}
