//! A palette section without a page of its own shows its best rows, and
//! its last row shows the rest in place.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::{App, Setup};

use crate::common::{launch, palette};

const PALETTE: &str = "Command palette";

fn buttons(harness: &Harness<'static, App>) -> Vec<String> {
    harness
        .query_all_by_role(Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .collect()
}

#[test]
fn show_all_expands_the_section_and_keeps_the_palette_open() {
    let mut harness = launch(Setup::default());
    palette(&mut harness, "e");

    let before = buttons(&harness);
    let show_all = before
        .iter()
        .find(|label| label.starts_with("Show all") && !label.contains(" in "))
        .expect("a section without a page")
        .clone();

    harness
        .get_by_role_and_label(Role::Button, &show_all)
        .click_accesskit();
    harness.run();

    let after = buttons(&harness);

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, PALETTE)
            .is_some(),
        "the palette stays open"
    );
    assert!(!after.contains(&show_all));
    assert!(after.len() > before.len());
}
