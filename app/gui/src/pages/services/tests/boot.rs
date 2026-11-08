use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::services::FileState;

use super::{ServicesPage, loaded, sshd_listed, unit};

#[test]
fn the_switch_says_it_starts_the_unit_at_boot() {
    let mut harness = sshd_listed().harness(ServicesPage::default());
    harness.run();

    let switch = harness.get_by_role_and_label(Role::CheckBox, "Start sshd.service at boot");
    let caption = harness.get_by_label("At boot");
    assert!(
        caption.rect().right() <= switch.rect().left(),
        "the caption sits left of the switch"
    );
}

#[test]
fn the_menu_comes_before_the_switch_as_in_the_app_catalog() {
    let mut harness = sshd_listed().harness(ServicesPage::default());
    harness.run();

    let menu = harness.get_by_label("Actions for sshd.service").rect();
    let switch = harness
        .get_by_role_and_label(Role::CheckBox, "Start sshd.service at boot")
        .rect();

    assert!(
        menu.right() <= switch.left(),
        "menu {menu:?} switch {switch:?}"
    );
}

#[test]
fn start_rests_on_a_running_unit() {
    let mut harness = sshd_listed().harness(ServicesPage::default());
    harness.run();

    harness.get_by_label("Actions for sshd.service").click();
    harness.run();

    let start = harness.get_by_role_and_label(Role::MenuItem, "Start");
    assert!(start.accesskit_node().is_disabled());
    let stop = harness.get_by_role_and_label(Role::MenuItem, "Stop");
    assert!(!stop.accesskit_node().is_disabled());
}

#[test]
fn stop_rests_on_a_stopped_unit() {
    let stopped = unit("cups.service", "CUPS", FileState::Enabled);
    let mut harness = loaded(vec![stopped]).harness(ServicesPage::default());

    harness.state_mut().page.filter.hide_inactive = false;

    harness.run();

    harness.get_by_label("Actions for cups.service").click();
    harness.run();

    let stop = harness.get_by_role_and_label(Role::MenuItem, "Stop");
    assert!(stop.accesskit_node().is_disabled());
}
