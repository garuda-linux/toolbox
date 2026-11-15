//! The next-step tiles under the greeting.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::painted_text;

use super::page;
use crate::testing::PageFixture;

const SETUP: &str = "Pick the apps you want and install them in one go";

#[test]
fn a_step_wraps_its_description_instead_of_cutting_it() {
    let harness = page(PageFixture::default());

    let shape = painted_text(&harness, SETUP).expect("the description is painted");
    assert!(!shape.galley.elided, "the description is cut");
    assert!(
        shape.galley.rows.len() <= 2,
        "{} lines",
        shape.galley.rows.len()
    );
}

#[test]
fn the_steps_of_one_line_are_as_tall() {
    let harness = page(PageFixture::default());

    let heights: Vec<f32> = ["Setup Assistant", "Maintenance", "Diagnostics"]
        .iter()
        .map(|name| {
            harness
                .get_by_role_and_label(Role::Link, name)
                .rect()
                .height()
        })
        .collect();

    assert!(
        heights
            .iter()
            .all(|height| (height - heights[0]).abs() < 0.5),
        "{heights:?}"
    );
}
