use egui::scroll_area::ScrollArea;
use egui::{Context, Rangef, RawInput, Rect, Shape, Theme, Ui, pos2, vec2};

use super::{
    GROOVE_WIDTH, HOVER_WIDTH, MIN_LENGTH, REST_WIDTH, STRIP, cross_span, show_scroll,
    show_scroll_hidden, show_scroll_rows, thumb_span,
};
use crate::palette::Palette;
use crate::theme::apply;

#[test]
fn handle_is_the_share_of_the_track_between_the_end_insets() {
    // A 100 px strip keeps 6 px at each end, so the track is 88 px and half
    // the content shows, which makes a 44 px handle.
    let strip = Rangef::new(0.0, 100.0);
    assert_eq!(thumb(strip, 100.0, 200.0, 0.0), (6.0, 50.0));
    assert_eq!(thumb(strip, 100.0, 200.0, 100.0), (50.0, 94.0));
    assert_eq!(thumb(strip, 100.0, 200.0, 50.0), (28.0, 72.0));
}

#[test]
fn handle_never_shrinks_below_the_minimum_length() {
    let strip = Rangef::new(0.0, 100.0);
    assert_eq!(thumb(strip, 100.0, 10_000.0, 0.0), (6.0, 6.0 + MIN_LENGTH));
}

#[test]
fn handle_fills_a_track_shorter_than_the_minimum() {
    let strip = Rangef::new(0.0, 22.0);
    assert_eq!(thumb(strip, 10.0, 1_000.0, 0.0), (6.0, 16.0));
}

#[test]
fn offsets_past_either_end_clamp_to_the_track() {
    let strip = Rangef::new(0.0, 100.0);
    assert_eq!(thumb(strip, 100.0, 200.0, -40.0), (6.0, 50.0));
    assert_eq!(thumb(strip, 100.0, 200.0, 500.0), (50.0, 94.0));
}

#[test]
fn handle_and_groove_centre_one_pixel_in_from_the_strip_edge() {
    let strip = Rangef::new(100.0, 100.0 + STRIP);
    assert_eq!(cross(strip, REST_WIDTH), (109.0, 113.0));
    assert_eq!(cross(strip, HOVER_WIDTH), (107.0, 115.0));
    assert_eq!(cross(strip, GROOVE_WIDTH), (107.0, 115.0));
}

#[test]
fn the_strip_is_darklys_21_pixels() {
    assert!((STRIP - 21.0).abs() < f32::EPSILON);
    assert!((REST_WIDTH - 4.0).abs() < f32::EPSILON);
    assert!((HOVER_WIDTH - 8.0).abs() < f32::EPSILON);
}

fn styled() -> Context {
    let ctx = Context::default();
    apply(
        &ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );
    ctx
}

#[test]
fn egui_keeps_the_strip_and_the_drag_but_draws_no_bar_of_its_own() {
    let scroll = styled().style_of(Theme::Dark).spacing.scroll;
    assert!((scroll.floating_allocated_width - STRIP).abs() < f32::EPSILON);
    assert!((scroll.bar_width - STRIP).abs() < f32::EPSILON);
    assert!((scroll.handle_min_length - MIN_LENGTH).abs() < f32::EPSILON);
    assert!(scroll.dormant_handle_opacity.abs() < f32::EPSILON);
    assert!(scroll.dormant_background_opacity.abs() < f32::EPSILON);
}

#[test]
fn bars_paint_a_rest_handle_only_where_content_overflows() {
    let ctx = styled();
    assert!(visible_handles(&ctx, 2).is_empty(), "nothing to scroll");

    let handles = visible_handles(&ctx, 80);
    assert_eq!(handles.len(), 1, "one vertical handle");
    assert!((handles[0].width() - REST_WIDTH).abs() < 0.01);
}

#[test]
fn a_hidden_area_paints_no_handle_however_far_it_overflows() {
    let ctx = styled();
    let handles = handles_of(&ctx, |ui| {
        show_scroll_hidden(ui, ScrollArea::vertical(), |ui| allocate_rows(ui, 80));
    });

    assert!(handles.is_empty(), "{handles:?}");
}

#[test]
fn rows_shown_through_show_scroll_rows_paint_their_handle() {
    let ctx = styled();
    let handles = handles_of(&ctx, |ui| {
        show_scroll_rows(ui, ScrollArea::vertical(), 14.0, 80, |ui, rows| {
            for _ in rows {
                ui.allocate_space(vec2(100.0, 14.0));
            }
        });
    });

    assert_eq!(handles.len(), 1, "{handles:?}");
}

fn thumb(strip: Rangef, viewport: f32, content: f32, offset: f32) -> (f32, f32) {
    let span = thumb_span(strip, viewport, content, offset);
    (span.min, span.max)
}

fn cross(strip: Rangef, width: f32) -> (f32, f32) {
    let span = cross_span(strip, width);
    (span.min, span.max)
}

fn visible_handles(ctx: &Context, rows: usize) -> Vec<Rect> {
    handles_of(ctx, |ui| {
        show_scroll(ui, ScrollArea::vertical(), |ui| allocate_rows(ui, rows));
    })
}

fn allocate_rows(ui: &mut Ui, rows: usize) {
    for _ in 0..rows {
        ui.allocate_space(vec2(100.0, 14.0));
    }
}

fn handles_of(ctx: &Context, show: impl FnMut(&mut Ui)) -> Vec<Rect> {
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0))),
        ..RawInput::default()
    };

    let mut output = ctx.run_ui(input, show);

    output.textures_delta.clear();

    output
        .shapes
        .iter()
        .filter_map(|clipped| {
            let Shape::Rect(rect) = &clipped.shape else {
                return None;
            };

            (rect.fill.a() > 0 && is_rest_handle(rect.rect)).then_some(rect.rect)
        })
        .collect()
}

fn is_rest_handle(rect: Rect) -> bool {
    (rect.width() - REST_WIDTH).abs() < 0.01
}

/// egui shrinks the area to its one short row, and keeps the bar's strip
/// right under it, so the handle sits there and not under the room the
/// area could have taken.
#[test]
fn a_shrunk_area_paints_its_sideways_handle_under_its_content() {
    let ctx = styled();
    let frame = || {
        ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0))),
                ..RawInput::default()
            },
            |ui| {
                show_scroll(ui, ScrollArea::horizontal(), |ui| {
                    ui.allocate_space(vec2(1000.0, 14.0));
                });
            },
        )
    };

    // The first frame learns that the row overflows, the second keeps the strip.
    frame().textures_delta.clear();
    let mut output = frame();

    output.textures_delta.clear();
    let handles: Vec<Rect> = output
        .shapes
        .iter()
        .filter_map(|clipped| {
            let Shape::Rect(rect) = &clipped.shape else {
                return None;
            };

            ((rect.rect.height() - REST_WIDTH).abs() < 0.01).then_some(rect.rect)
        })
        .collect();

    // Under the row and its strip, not near the bottom of the 300 point screen.
    assert!(
        handles
            .iter()
            .any(|handle| handle.top() > 14.0 && handle.bottom() < 2.0_f32.mul_add(STRIP, 14.0)),
        "no handle in the strip under the row: {handles:?}"
    );
}
