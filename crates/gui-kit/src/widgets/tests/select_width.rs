use egui_kittest::Harness;

use super::super::Select;
use crate::testing::{harness, painted_text};

const LONG: &str = "Deutsch (Deutschland), de_DE.UTF-8";

fn shown(options: &'static [&'static str], chosen: usize) -> Harness<'static, f32> {
    let mut harness = harness(900.0, 120.0).build_ui_state(
        move |ui, width: &mut f32| {
            *width = Select::fitting_width(ui, options.iter().copied());
            _ = Select::new("Language", options)
                .selected(Some(chosen))
                .width(*width)
                .show(ui);
        },
        0.0,
    );

    harness.run();
    harness
}

#[test]
fn a_fitting_width_shows_the_longest_option_whole() {
    let harness = shown(&["English", LONG], 1);
    let text = painted_text(&harness, LONG).expect("the option is painted");

    assert!(!text.galley.elided, "cut at {}", harness.state());
}

#[test]
fn a_fitting_width_stops_at_the_widest_open_list() {
    let narrow = shown(&["English", "x"], 0);
    let short = *narrow.state();

    let harness = shown(
        &[
            LONG,
            "a name far too long for any dropdown to hold, a name far too long for any dropdown to hold",
        ],
        0,
    );

    assert!(short < 100.0, "{short}");
    assert!((harness.state() - super::super::select::LIST_MAX_WIDTH).abs() < 0.5);
}
