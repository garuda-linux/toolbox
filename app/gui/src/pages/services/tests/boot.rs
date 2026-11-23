use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::services::{FileState, Shown};

use super::{ServicesPage, loaded, sshd, sshd_listed, unit};

/// One caption names the whole switch column, so no row repeats it.
#[test]
fn one_caption_above_the_switches_says_they_start_units_at_boot() {
    let avahi = unit("avahi-daemon.service", "Avahi", FileState::Enabled);
    let mut harness = loaded(vec![avahi, sshd()]).harness(ServicesPage::default());
    harness.state_mut().page.filter.shown = Shown::All;
    harness.run();

    assert_eq!(harness.query_all_by_label("At boot").count(), 1);

    let caption = harness.get_by_label("At boot").rect();
    let switch = harness
        .get_by_role_and_label(Role::CheckBox, "Start avahi-daemon.service at boot")
        .rect();

    assert!(caption.bottom() <= switch.top(), "{caption:?} {switch:?}");

    // The words are centred on the track, one pixel of rounding apart.
    assert!(
        (caption.center().x - switch.center().x).abs() < 1.5
            || (caption.right() - switch.right()).abs() < 1.5,
        "the caption heads the switch column: {caption:?} {switch:?}"
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

    harness.state_mut().page.filter.shown = Shown::All;

    harness.run();

    harness.get_by_label("Actions for cups.service").click();
    harness.run();

    let stop = harness.get_by_role_and_label(Role::MenuItem, "Stop");
    assert!(stop.accesskit_node().is_disabled());
}

/// The file tag's meaning shows for the focused row too, not only under
/// the pointer.
#[test]
fn a_focused_row_says_what_its_file_tag_means() {
    let journal = unit("systemd-journald.service", "Journal", FileState::Static);
    let mut harness = loaded(vec![journal]).harness(ServicesPage::default());
    harness.state_mut().page.filter.shown = Shown::All;
    harness.run();

    let hint = "Cannot be enabled. It starts when another service or the system needs it.";
    assert!(harness.query_by_label(hint).is_none());

    harness
        .query_all_by_role(Role::ListItem)
        .next()
        .expect("the row")
        .focus();
    harness.run();

    assert!(harness.query_by_label(hint).is_some());
}
