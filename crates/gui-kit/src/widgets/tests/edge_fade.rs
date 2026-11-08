//! A scrolled list's entries fade toward an edge with more beyond it, and
//! only there.

use egui::{ScrollArea, Ui};

use super::super::EdgeFade;
use crate::testing::{harness, painted_text};

const ROW: f32 = 32.0;
const ROWS: u8 = 20;

/// A list of `ROWS` rows in a 200 tall window, at its top or at its end.
fn list(at_end: bool) -> egui_kittest::Harness<'static> {
    let mut harness = harness(200.0, 200.0).build_ui(move |ui: &mut Ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ScrollArea::vertical()
            .stick_to_bottom(at_end)
            .show(ui, |ui| {
                let fade = EdgeFade::of(ui, f32::from(ROWS) * ROW, ROW);
                for row in 0..ROWS {
                    fade.show(ui, ROW, |ui| {
                        ui.add_sized([100.0, ROW], egui::Label::new(format!("Row {row}")));
                    });
                }
            });
    });

    harness.run_steps(3);
    harness
}

fn opacity(harness: &egui_kittest::Harness<'static>, row: u8) -> f32 {
    painted_text(harness, &format!("Row {row}"))
        .map_or(0.0, |text| f32::from(text.fallback_color.a()) / 255.0)
}

#[test]
fn an_unscrolled_list_fades_only_at_its_bottom() {
    let harness = list(false);

    assert!(
        (opacity(&harness, 0) - 1.0).abs() < 0.01,
        "the top row fades"
    );
    assert!(opacity(&harness, 2) > 0.99, "a row in the middle fades");
    assert!(
        opacity(&harness, 5) < 0.9,
        "the cut row at the bottom is whole"
    );
}

#[test]
fn a_list_scrolled_to_its_end_fades_only_at_its_top() {
    let harness = list(true);

    assert!(
        opacity(&harness, ROWS - 1) > 0.99,
        "the last row fades with nothing below it"
    );
    let first_seen = (0..ROWS)
        .find(|row| painted_text(&harness, &format!("Row {row}")).is_some())
        .expect("a row in sight");

    assert!(
        opacity(&harness, first_seen) < 0.9,
        "the cut top row is whole"
    );
}
