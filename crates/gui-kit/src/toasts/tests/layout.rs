use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{MAX_WIDTH, ToastKind, Toasts};
use super::{TEXTS, shown, with};
use crate::testing::{cap_off_centre, filled_rect, harness, tinted_rects};
use crate::tokens;
use crate::widgets::Tone;

const SHORT: &str = "A step failed. Open the Console to see what happened.";
const LONG: &str = "A step failed while another program held the package database, \
                    so nothing changed. Open the Console to see what happened.";

/// The first note's area, as egui keeps it.
fn note(harness: &Harness<'static, Toasts>) -> Rect {
    let id = egui::Id::unique(("toolbox_gui_kit::toast", 0_u64));
    harness
        .ctx
        .memory(|memory| memory.area_rect(id))
        .expect("no note is shown")
}

fn tile(harness: &Harness<'static, Toasts>, tone: Tone) -> Rect {
    let tokens = tokens::current(&harness.ctx);
    let (tile, _) = tone.status_mark(&tokens);
    filled_rect(harness, tile.fill).expect("the note paints no status tile")
}

fn close(harness: &Harness<'static, Toasts>) -> Rect {
    harness.get_by_role_and_label(Role::Button, "Close").rect()
}

#[test]
fn a_short_sentence_stays_on_one_line_and_the_note_grows_for_it() {
    let one_word = shown(with(ToastKind::Error, "Failed"));
    let line = one_word.get_by_label("Failed").rect().height();

    let harness = shown(with(ToastKind::Error, SHORT));
    let text = harness.get_by_label(SHORT).rect();

    assert!(
        (text.height() - line).abs() < 0.5,
        "{SHORT:?} wraps: {text:?}"
    );
    assert!(
        note(&harness).width() <= MAX_WIDTH + 0.5,
        "{:?}",
        note(&harness)
    );
    assert!(note(&one_word).width() < note(&harness).width());
}

#[test]
fn a_long_text_wraps_at_the_widest_note() {
    let harness = shown(with(ToastKind::Error, LONG));

    let width = note(&harness).width();
    assert!((width - MAX_WIDTH).abs() < 0.5, "the note is {width} wide");
    assert!(width <= MAX_WIDTH + 0.5, "the note is {width} wide");
}

#[test]
fn a_note_keeps_inside_a_narrow_window() {
    let mut harness = harness(320.0, 300.0).build_ui_state(
        |ui, toasts: &mut Toasts| toasts.show(ui.ctx(), ui.max_rect(), TEXTS),
        with(ToastKind::Error, SHORT),
    );

    harness.run_steps(2);

    let note = note(&harness);
    assert!(note.left() >= 0.0 && note.right() <= 320.0, "{note:?}");
}

#[test]
fn the_tile_the_text_and_close_share_one_middle() {
    for text in [SHORT, LONG] {
        let harness = shown(with(ToastKind::Error, text));
        let tile = tile(&harness, Tone::Error);
        let block = harness.get_by_label(text).rect();
        let close = close(&harness);

        let middle = tile.center().y;
        assert!(
            (block.center().y - middle).abs() < 0.5,
            "{block:?} {tile:?}"
        );
        assert!(
            (close.center().y - middle).abs() < 0.5,
            "{close:?} {tile:?}"
        );
    }
}

#[test]
fn one_line_sits_on_the_tile_middle_by_its_capitals() {
    let harness = shown(with(ToastKind::Error, SHORT));
    let tile = tile(&harness, Tone::Error);

    let off = cap_off_centre(&harness, SHORT, tile).expect("the text is painted");

    // Half a pixel of rounding either way, as every control allows.
    assert!(
        off.abs() < 0.75,
        "the capitals sit {off} off the tile's middle"
    );
}

#[test]
fn the_padding_is_the_same_all_round() {
    let harness = shown(with(ToastKind::Error, SHORT));
    let (note, tile, close) = (note(&harness), tile(&harness, Tone::Error), close(&harness));
    let content = tile.union(close);

    let sides = [
        content.left() - note.left(),
        note.right() - content.right(),
        content.top() - note.top(),
        note.bottom() - content.bottom(),
    ];

    for side in sides {
        assert!((side - sides[0]).abs() < 0.5, "{sides:?}");
    }
}

#[test]
fn each_kind_marks_itself_with_its_status_tile_and_colour() {
    for (kind, tone) in [
        (ToastKind::Error, Tone::Error),
        (ToastKind::Info, Tone::Accent),
    ] {
        let harness = shown(with(kind, "Copied"));
        let tokens = tokens::current(&harness.ctx);
        let (mark, _) = tone.status_mark(&tokens);
        let square = tile(&harness, tone);

        let glyph = tinted_rects(&harness, mark.ink)
            .into_iter()
            .find(|glyph| square.contains_rect(*glyph));

        assert!(
            glyph.is_some(),
            "{kind:?}: no glyph in {:?} on the tile",
            mark.ink
        );
        assert!((square.width() - square.height()).abs() < 0.5, "{square:?}");
    }
}
