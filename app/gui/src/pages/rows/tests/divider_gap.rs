//! The room between a divider and the capitals of the row below it, read
//! from the rendered picture, against the room under the row's last
//! baseline.

use egui::{Label, Rect, Ui};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::testing::{harness, lines_in, painted_text};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{card, card_row, rich, title_line};

use crate::testing::letters_top;

/// Off by at most this, two gaps count as one.
const SAME: f32 = 1.0;

/// A title without ascenders, so its first ink is the capitals' top.
const BELOW: &str = "Swap";
const LAST_LINE: &str = "Asks the admin first";

fn row(ui: &mut Ui, (name, last): (&str, &str)) {
    let tokens = tokens::current(ui.ctx());
    title_line(ui, false, |ui| {
        ui.label(rich(ui, name, Weight::SemiBold, BODY, tokens.text));
    });

    ui.add(Label::new(rich(
        ui,
        last,
        Weight::Regular,
        SMALL,
        tokens.secondary,
    )));
}

#[test]
fn a_row_below_a_divider_has_its_last_baselines_room_clear_above_its_capitals() {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        |ui, below: &mut Rect| {
            card(ui, |ui| {
                card_row(ui, true, |ui| row(ui, ("Zram", "The first row")));
                *below = card_row(ui, false, |ui| row(ui, (BELOW, LAST_LINE)))
                    .response
                    .rect;
            });
        },
        Rect::NOTHING,
    );

    harness.run();
    let shape = painted_text(&harness, BELOW).expect("the title is painted");
    let title = shape.galley.rect.translate(shape.pos.to_vec2());
    let capitals = letters_top(&mut harness, title).expect("the title shows light letters");
    let outline = tokens::current(&harness.ctx).outline;
    let divider = lines_in(&harness, outline)
        .into_iter()
        .map(|[start, _]| start.y)
        .filter(|y| *y < capitals)
        .fold(f32::MIN, f32::max);

    // The last line has no descenders, so its ink ends on its baseline.
    let last = painted_text(&harness, LAST_LINE).expect("the last line is painted");
    let baseline = last.galley.mesh_bounds.bottom() + last.pos.y;
    let row = *harness.state();

    // The divider is a line one pixel thick, centred on `divider`.
    let above = capitals - (divider + 0.5);
    let below = row.bottom() - baseline;
    assert!(
        (above - below).abs() <= SAME,
        "{above} clear above the capitals, {below} under the last baseline"
    );
}
