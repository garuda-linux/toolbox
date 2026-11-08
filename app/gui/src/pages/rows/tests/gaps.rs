//! The room above a card row's text and below it, read from the
//! rendered picture: the row's edge to the top of the title's capitals
//! above, the last line's box to the row's edge below.

use egui::{Label, Rect, Ui};
use egui_kittest::Harness;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::testing::{harness, painted_text};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{
    ButtonKind, card, card_row, paint_letter_tile, pill_button, rich, split_row, switch, title_line,
};

use super::text_rect;
use crate::pages::rows::{PICTURE_RADIUS, picture_slot};
use crate::testing::{drawing_top, letters, letters_top};

/// Off by at most this, two gaps count as one.
const SAME: f32 = 1.0;

/// Titles without ascenders, so their first ink is the capitals' top,
/// one without descenders, so its last ink is their bottom, and last
/// lines without descenders, so their ink ends on their baseline.
const LINES: &str = "Zram";
const ALONE: &str = "Users";
const PICTURED: &str = "Snapper";
const LAST_LINE: &str = "Asks the admin first";
const PICTURE_LINE: &str = "Restores the last state";

#[derive(Clone, Copy)]
struct Rows {
    lines: Rect,
    alone: Rect,
    pictured: Rect,
    picture: Rect,
}

fn small(ui: &mut Ui, text: &str) {
    let ink = tokens::current(ui.ctx()).secondary;
    ui.add(Label::new(rich(ui, text, Weight::Regular, SMALL, ink)));
}

fn title(ui: &mut Ui, text: &str) {
    let ink = tokens::current(ui.ctx()).text;
    title_line(ui, false, |ui| {
        ui.label(rich(ui, text, Weight::SemiBold, BODY, ink));
    });
}

fn three_lines(ui: &mut Ui) {
    title(ui, LINES);
    small(ui, "/etc/systemd/zram-generator.conf");
    small(ui, LAST_LINE);
}

fn pictured(ui: &mut Ui) -> Rect {
    ui.horizontal_top(|ui| {
        let at = picture_slot(ui, false);
        paint_letter_tile(ui, at, PICTURE_RADIUS, PICTURED, "snapper");
        ui.vertical(|ui| {
            title(ui, PICTURED);
            small(ui, PICTURE_LINE);
        });
        at
    })
    .inner
}

fn button(ui: &mut Ui) {
    _ = pill_button(ui, "Edit", ButtonKind::Soft);
}

fn drawn() -> (Harness<'static, Rows>, Rows) {
    let mut harness = harness(800.0, 400.0).build_ui_state(
        |ui, rows: &mut Rows| {
            card(ui, |ui| {
                rows.lines = card_row(ui, true, |ui| {
                    split_row(ui, 80.0, three_lines, button);
                })
                .response
                .rect;

                rows.alone = card_row(ui, false, |ui| {
                    split_row(ui, 80.0, |ui| title(ui, ALONE), button);
                })
                .response
                .rect;

                let drawn = card_row(ui, false, |ui| {
                    split_row(ui, 40.0, pictured, |ui| switch(ui, true, PICTURED, true)).0
                });

                rows.pictured = drawn.response.rect;
                rows.picture = drawn.inner;
            });
        },
        Rows {
            lines: Rect::NOTHING,
            alone: Rect::NOTHING,
            pictured: Rect::NOTHING,
            picture: Rect::NOTHING,
        },
    );

    harness.run();
    let rows = *harness.state();
    (harness, rows)
}

fn gaps(harness: &mut Harness<'_, Rows>, row: Rect, title: &str, last: &str) -> [f32; 3] {
    let title = text_rect(harness, title);
    let capitals = letters_top(harness, title).expect("the title shows light letters");
    let shape = painted_text(harness, last).unwrap_or_else(|| panic!("{last} is painted"));

    // The last line has no descenders, so its ink ends on its baseline.
    let baseline = shape.galley.mesh_bounds.bottom() + shape.pos.y;
    let last = text_rect(harness, last);
    [
        capitals - row.top(),
        row.bottom() - baseline,
        row.bottom() - last.bottom(),
    ]
}

#[test]
fn a_rows_capitals_have_as_much_room_above_as_its_last_baseline_below() {
    let (mut harness, rows) = drawn();
    let padding = tokens::Sizes::DEFAULT.row_padding.y;
    let [above, below, box_below] = gaps(&mut harness, rows.lines, LINES, LAST_LINE);

    assert!(
        (above - below).abs() <= SAME,
        "the capitals start {above} below the row's top, the last baseline {below} above its \
         bottom"
    );
    assert!(
        (box_below - padding).abs() <= SAME,
        "the last line's box ends {box_below} above the row's bottom"
    );
}

/// The control is taller than one line, so the title's capitals sit on
/// its middle, as a control's own text does, with as much room above as
/// below them and more than the padding.
#[test]
fn a_single_line_beside_a_control_has_as_much_room_above_its_capitals_as_below() {
    let (mut harness, rows) = drawn();
    let title = text_rect(&harness, ALONE);
    let (top, bottom) = letters(&mut harness, title).expect("the title shows light letters");
    let (above, below) = (top - rows.alone.top(), rows.alone.bottom() - bottom);

    assert!(
        (above - below).abs() <= SAME,
        "the capitals start {above} below the row's top and end {below} above its bottom"
    );
}

#[test]
fn a_picture_starts_at_the_capitals_as_far_down_as_the_last_baseline_is_up() {
    let (mut harness, rows) = drawn();
    let [above, below, _] = gaps(&mut harness, rows.pictured, PICTURED, PICTURE_LINE);
    let picture = drawing_top(&mut harness, rows.picture).expect("the tile is painted");

    assert!(
        (picture - rows.pictured.top() - above).abs() <= SAME,
        "the tile starts at {picture}, the capitals {above} below the row's top {}",
        rows.pictured.top()
    );
    assert!(
        (above - below).abs() <= SAME,
        "the capitals start {above} below the row's top, the last baseline {below} above its \
         bottom"
    );
}
