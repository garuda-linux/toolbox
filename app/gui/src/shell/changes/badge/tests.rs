use egui::Ui;
use toolbox_gui_kit::testing::harness;

use super::badge;

fn width_of(count: &'static str) -> f32 {
    let mut harness = harness(400.0, 200.0).build_ui_state(
        move |ui: &mut Ui, width: &mut f32| *width = badge(ui, count).rect.width(),
        0.0,
    );

    harness.run();
    *harness.state()
}

#[test]
fn the_badge_keeps_its_width_for_one_to_two_digit_counts() {
    let widths = ["1", "9", "10", "12", "99"].map(width_of);

    for width in widths {
        assert_eq!(width, widths[0], "widths {widths:?}");
    }
}

#[test]
fn the_badge_grows_past_two_digits() {
    assert!(width_of("100") > width_of("99"));
}
