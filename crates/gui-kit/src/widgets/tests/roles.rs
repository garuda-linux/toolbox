use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{NavLook, Select, nav_row};
use crate::icons;
use crate::testing::harness;

#[test]
fn the_open_page_is_a_selected_tab_not_a_pressed_button() {
    let mut harness = harness(300.0, 200.0).build_ui(|ui| {
        let look = NavLook {
            expanded: true,
            height: 32.0,
        };

        _ = nav_row(ui, icons::GO_HOME, "Home", true, look);
        _ = nav_row(ui, icons::GO_HOME, "Kernels", false, look);
    });

    harness.run();

    let open = harness.get_by_label("Home");
    assert_eq!(open.accesskit_node().role(), Role::Tab);
    assert_eq!(open.accesskit_node().is_selected(), Some(true));
    assert_eq!(open.accesskit_node().toggled(), None::<Toggled>);

    let other = harness.get_by_label("Kernels");
    assert_eq!(other.accesskit_node().is_selected(), Some(false));
}

#[test]
fn a_select_lists_its_options_as_options_with_the_chosen_one_selected() {
    let mut harness = harness(400.0, 300.0).build_ui(|ui| {
        let options = ["Name", "Size"];
        _ = Select::new("Sort by", &options).selected(Some(1)).show(ui);
    });

    harness.run();
    harness.get_by_label("Sort by").click();
    harness.run();

    let chosen = harness.get_by_label("Size");
    assert_eq!(chosen.accesskit_node().role(), Role::ListBoxOption);
    assert_eq!(chosen.accesskit_node().is_selected(), Some(true));
    assert_eq!(chosen.accesskit_node().toggled(), None::<Toggled>);
}
