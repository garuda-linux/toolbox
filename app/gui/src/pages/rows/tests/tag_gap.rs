//! A tag beside a card row's title: above the next line, its words
//! inside it, the line below where it is without a tag.

use egui::{Label, Rect, Ui};
use egui_kittest::Harness;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::testing::{filled_rect, harness, painted_text};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{Tone, card, card_row, rich, tag, title_line};

use super::text_rect;

/// Off by at most this, two edges count as one.
const SAME: f32 = 1.0;

/// Titles without ascenders, so their first ink is the capitals' top.
const TAGGED: &str = "Zram";
const PLAIN: &str = "Swap";
/// A tag with a descender, which must stay inside the pill.
const TAG: &str = "Dependency";
const TAGGED_LINE: &str = "The line under the tagged title";
const PLAIN_LINE: &str = "The line under the plain title";

#[derive(Clone, Copy)]
struct Rows {
    tagged: Rect,
    plain: Rect,
}

fn row(ui: &mut Ui, (name, below, tagged): (&str, &str, bool)) {
    let tokens = tokens::current(ui.ctx());
    title_line(ui, false, |ui| {
        ui.label(rich(ui, name, Weight::SemiBold, BODY, tokens.text));
        if tagged {
            tag(ui, TAG, Tone::Accent);
        }
    });

    let line = rich(ui, below, Weight::Regular, SMALL, tokens.secondary);
    ui.add(Label::new(line));
}

fn drawn() -> (Harness<'static, Rows>, Rows) {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        |ui, rows: &mut Rows| {
            card(ui, |ui| {
                rows.tagged = card_row(ui, true, |ui| row(ui, (TAGGED, TAGGED_LINE, true)))
                    .response
                    .rect;
                rows.plain = card_row(ui, false, |ui| row(ui, (PLAIN, PLAIN_LINE, false)))
                    .response
                    .rect;
            });
        },
        Rows {
            tagged: Rect::NOTHING,
            plain: Rect::NOTHING,
        },
    );

    harness.run();
    let rows = *harness.state();
    (harness, rows)
}

fn pill(harness: &Harness<'_, Rows>) -> Rect {
    let fill = tokens::current(&harness.ctx).accent_soft;
    filled_rect(harness, fill).expect("the tag is painted")
}

#[test]
fn a_tag_ends_above_the_next_line() {
    let (harness, _) = drawn();
    let next = text_rect(&harness, TAGGED_LINE);
    let pill = pill(&harness);

    assert!(
        pill.bottom() < next.top(),
        "the tag ends at {}, the next line starts at {}",
        pill.bottom(),
        next.top()
    );
}

#[test]
fn a_tags_words_stay_inside_it() {
    let (harness, _) = drawn();
    let pill = pill(&harness);
    let shape = painted_text(&harness, TAG).expect("the tag's words are painted");
    let ink = shape.galley.mesh_bounds.translate(shape.pos.to_vec2());

    // A glyph's quad holds a pixel of soft edge around its ink.
    assert!(
        pill.expand(SAME).contains_rect(ink),
        "the words' ink {ink:?} leaves the tag {pill:?}"
    );
}

#[test]
fn a_tag_does_not_move_the_line_below() {
    let (harness, rows) = drawn();
    let tagged = text_rect(&harness, TAGGED_LINE).top() - rows.tagged.top();
    let plain = text_rect(&harness, PLAIN_LINE).top() - rows.plain.top();

    assert!(
        (tagged - plain).abs() < 0.5,
        "the line starts {tagged} into the tagged row, {plain} into the plain one"
    );
}
