use egui::accesskit::{Role, Toggled};
use egui::vec2;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_plan::sdboot::LoaderSetting;

use super::{LOADER_CONF, Shown, image_menu, loader, sent, shown};

/// A narrow window beside the open sidebar, where a 340 point control took
/// more than half of its row.
const NARROW: f32 = 640.0;

fn segment(harness: &mut Shown, group: &str, option: &str) {
    harness
        .get_by_role_and_label(Role::RadioGroup, group)
        .get_by_role_and_label(Role::RadioButton, option)
        .scroll_to_me();
    harness.run();
    harness
        .get_by_role_and_label(Role::RadioGroup, group)
        .get_by_role_and_label(Role::RadioButton, option)
        .click();
    harness.run();
}

#[test]
fn a_switch_takes_on_off_or_systemd_boots_own_default() {
    let mut harness = shown(image_menu(LOADER_CONF));

    segment(&mut harness, "Edit entries at boot", "Off");

    assert_eq!(sent(&harness), [loader(LoaderSetting::Editor(Some(false)))]);
}

/// The image's `loader.conf` with `editor yes` added: On is picked. The
/// firmware entry is unset there, so a click on its Default sends nothing.
#[test]
fn a_tri_state_setting_is_a_segment_on_its_current_choice() {
    let conf = format!("{LOADER_CONF}editor yes\n");
    let mut harness = shown(image_menu(&conf));

    let on = harness
        .get_by_role_and_label(Role::RadioGroup, "Edit entries at boot")
        .get_by_role_and_label(Role::RadioButton, "On")
        .accesskit_node()
        .toggled();

    assert_eq!(on, Some(Toggled::True));

    segment(&mut harness, "Edit entries at boot", "Default");
    segment(&mut harness, "Firmware settings entry", "Default");

    assert_eq!(sent(&harness), [loader(LoaderSetting::Editor(None))]);
}

#[test]
fn a_wide_control_takes_at_most_two_fifths_of_a_narrow_row() {
    let mut harness = shown(image_menu(LOADER_CONF));
    harness.set_size(vec2(NARROW, 900.0));
    harness.run();

    for select in ["Default entry", "Menu timeout"] {
        let width = harness
            .get_by_role_and_label(Role::ComboBox, select)
            .rect()
            .width();

        assert!(width <= NARROW * 0.4, "{select} is {width} wide");
    }
}
