use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::{painted_text_within, rims_around};

use super::{page, system};
use crate::pages::Refusals;
use crate::testing::rect_of;

const NAME: &str = "American English (United States), en_US.UTF-8";

/// The page beside the open sidebar of a 1024 point window.
const BESIDE_SIDEBAR: f32 = 776.0;

#[test]
fn the_system_language_shows_its_whole_name() {
    let mut harness = page(system(), Refusals::default());
    harness.set_size(vec2(BESIDE_SIDEBAR, 900.0));
    harness.run();

    let pill = rect_of(&harness, Role::ComboBox, "System language");
    let shown = painted_text_within(&harness, NAME, pill).expect("the pill shows the language");
    assert!(!shown.galley.elided, "the pill cuts {NAME}");
}

#[test]
fn a_narrow_window_keeps_the_system_language_inside_the_page() {
    let width = 420.0;
    let mut harness = page(system(), Refusals::default());
    harness.set_size(vec2(width, 900.0));
    harness.run();

    let pill = rect_of(&harness, Role::ComboBox, "System language");
    assert!(pill.right() <= width, "the pill ends at {}", pill.right());
}

#[test]
fn queue_all_missing_has_no_rim_as_the_kernels_banner_buttons() {
    let mut harness = page(system(), Refusals::default());
    harness
        .get_by_role_and_label(Role::RadioButton, "Language packs")
        .click();
    harness.run();

    let button = rect_of(&harness, Role::Button, "Queue all missing");
    let rims = rims_around(&harness, button.center());
    assert!(
        rims.iter().all(|width| *width == 0.0),
        "rims of {rims:?} round Queue all missing"
    );
}
