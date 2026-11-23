use egui::epaint::RectShape;
use egui::{Rect, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{MAX_WIDTH, ToastKind, Toasts};
use super::{TEXTS, shown, with};
use crate::testing::{filled_rect, harness, painted_text};
use crate::tokens;

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

#[test]
fn a_short_sentence_stays_on_one_line_and_the_note_grows_for_it() {
    let one_word = shown(with(ToastKind::Info, "Failed"));
    let line = one_word.get_by_label("Failed").rect().height();

    let harness = shown(with(ToastKind::Info, SHORT));
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
    let harness = shown(with(ToastKind::Info, LONG));

    let width = note(&harness).width();
    assert!((width - MAX_WIDTH).abs() < 0.5, "the note is {width} wide");
}

#[test]
fn a_note_keeps_inside_a_narrow_window() {
    let mut harness = harness(320.0, 300.0).build_ui_state(
        |ui, toasts: &mut Toasts| toasts.show(ui.ctx(), ui.max_rect(), TEXTS),
        with(ToastKind::Info, SHORT),
    );

    harness.run_steps(2);

    let note = note(&harness);
    assert!(note.left() >= 0.0 && note.right() <= 320.0, "{note:?}");
}

#[test]
fn the_padding_is_kirigamis_large_spacing_all_round() {
    let harness = shown(with(ToastKind::Info, SHORT));
    let note = note(&harness);
    let text = harness.get_by_label(SHORT).rect();

    let sides = [
        text.left() - note.left(),
        note.right() - text.right(),
        text.top() - note.top(),
        note.bottom() - text.bottom(),
    ];

    for side in sides {
        assert!((side - 8.0).abs() < 1.0, "{sides:?}");
    }
}

#[test]
fn a_note_is_the_complementary_set_at_ninety_percent_with_its_own_text() {
    let harness = shown(with(ToastKind::Info, SHORT));
    let tokens = tokens::current(&harness.ctx);

    let fill = filled_rect(&harness, tokens.roles.complementary.gamma_multiply(0.9));
    assert!(fill.is_some(), "no note painted in the complementary set");

    let text = painted_text(&harness, SHORT).expect("the text is painted");
    assert_eq!(text.fallback_color, tokens.roles.complementary_text);
}

/// The rectangles of `shape`, looking inside the groups a shadow is painted in.
fn rects_within(shape: &Shape) -> Vec<RectShape> {
    if let Shape::Rect(rect) = shape {
        return vec![rect.clone()];
    }

    if let Shape::Vec(shapes) = shape {
        return shapes.iter().flat_map(rects_within).collect();
    }

    Vec::new()
}

/// Darkly's popup shadow, two layers: 5 px down and 20 px soft at 44 % of
/// the shadow shade, and 3 px up and 10 px soft at 24 %.
#[test]
fn a_note_casts_darklys_two_layer_popup_shadow() {
    let harness = shown(with(ToastKind::Info, SHORT));
    let tokens = tokens::current(&harness.ctx);
    let note = note(&harness);

    for (blur, drop, share) in [(20.0, 5.0, 0.44), (10.0, -3.0, 0.24)] {
        let colour = tokens.shadow_shade.gamma_multiply(share);
        let found = harness.output().shapes.iter().any(|clipped| {
            rects_within(&clipped.shape).into_iter().any(|rect| {
                rect.fill == colour
                    && (rect.blur_width - blur).abs() < f32::EPSILON
                    && (rect.rect.center().y - note.center().y - drop).abs() < 0.01
            })
        });

        assert!(found, "no popup shadow {blur} px soft, {drop} px down");
    }
}
