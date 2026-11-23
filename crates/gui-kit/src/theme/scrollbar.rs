//! Darkly's scroll bars, painted over the bars egui keeps.
//!
//! egui still owns the wheel, the keyboard, the drag and the reserved strip,
//! as its bars take the strip width as their hit area. Its own bars are
//! drawn invisible (see [`super::write`]), so this pass adds the handle and
//! the groove in Darkly's sizes and colours. A call site paints it once
//! after each `ScrollArea::show`.

#[cfg(test)]
mod tests;

use egui::scroll_area::ScrollAreaOutput;
use egui::{Color32, Context, CornerRadius, Rangef, Rect, Ui, lerp, pos2};

use crate::tokens::{self, Tokens};

/// The strip a bar takes beside the content: Darkly's `ScrollBar_Extend`
/// (darkly.h:97).
pub(super) const STRIP: f32 = 21.0;
/// The handle at rest and under the pointer: Darkly's
/// `ScrollBar_SliderWidth` (darkly.h:98) halved and then whole
/// (darklystyle.cpp:6387-6399).
const REST_WIDTH: f32 = 4.0;
const HOVER_WIDTH: f32 = 8.0;
/// The groove, the full `ScrollBar_SliderWidth` (darklystyle.cpp:8150).
const GROOVE_WIDTH: f32 = 8.0;
/// Darkly's `ScrollBar_MinSliderHeight` (darkly.h:99).
pub(super) const MIN_LENGTH: f32 = 20.0;
/// Darkly's `ScrollBar_NoButtonHeight`, (21 - 8) / 2 rounded down, keeps the
/// handle off the strip's ends.
const END_INSET: f32 = 6.0;
/// The strip's near edge takes one pixel, which Darkly leaves to the thin
/// separator (darklystyle.cpp:6387, 8143), so handle and groove centre one
/// pixel in.
const EDGE: f32 = 1.0;
/// Breeze's handle is the text at half alpha (darklyhelper.cpp:355).
const HANDLE_ALPHA: f32 = 0.5;
/// The handle's alpha share at rest, 0.7 of its colour (darklystyle.cpp:6441).
/// Hovering the strip takes it to the full colour.
const REST_SHARE: f32 = 0.7;
/// The groove's alpha over the text (darklystyle.cpp:8160).
const GROOVE_ALPHA: f32 = 0.3;
/// The fade into the hover look, in seconds.
const TIME: f32 = 0.25;

/// Paints the handle and, on hover, the groove of each bar the content
/// overflows. Call it after the `ScrollArea` that produced `output`.
pub fn paint_bars<R>(ui: &Ui, output: &ScrollAreaOutput<R>) {
    let ctx = ui.ctx();
    let tokens = tokens::current(ctx);
    let focused = is_focused(ctx, output);
    let time = if super::reduced_motion(ctx) {
        0.0
    } else {
        TIME
    };

    for along in [0, 1] {
        let viewport = output.inner_rect.size()[along];
        let content = output.content_size[along];
        if content <= viewport {
            continue;
        }

        let strip = strip(output.inner_rect, along);
        let hovered = ui.rect_contains_pointer(strip);
        let hover_id = output.id.with(("darkly_bar", along));
        let groove_t = ctx.animate_bool_with_time(hover_id, hovered, time);
        let width = lerp(REST_WIDTH..=HOVER_WIDTH, groove_t);

        let (across, along_span) = spans(strip, along);
        let handle = thumb_span(
            along_span,
            viewport,
            content,
            output.state.clamped_offset()[along],
        );

        let painter = ui.painter();

        if groove_t > 0.0 {
            painter.rect_filled(
                rect(along, along_span, cross_span(across, GROOVE_WIDTH)),
                pill(GROOVE_WIDTH),
                tokens.text.gamma_multiply(GROOVE_ALPHA * groove_t),
            );
        }

        painter.rect_filled(
            rect(along, handle, cross_span(across, width)),
            pill(width),
            handle_colour(&tokens, hovered, focused, groove_t),
        );
    }
}

/// The colour of the handle: the hover colour over the strip, the focus
/// colour while the scrolled content has focus, else the text at half alpha.
/// The alpha share rises from 0.7 at rest to the full colour under the pointer.
fn handle_colour(tokens: &Tokens, hovered: bool, focused: bool, groove_t: f32) -> Color32 {
    let base = if hovered {
        tokens.decoration_hover
    } else if focused {
        tokens.decoration_focus
    } else {
        tokens.text.gamma_multiply(HANDLE_ALPHA)
    };

    base.gamma_multiply((1.0 - REST_SHARE).mul_add(groove_t, REST_SHARE))
}

/// The strip beside the content for an axis, at the bottom for a horizontal
/// bar and at the right for a vertical one, as egui places its own.
fn strip(inner: Rect, along: usize) -> Rect {
    if along == 1 {
        Rect::from_min_max(
            pos2(inner.max.x, inner.min.y),
            pos2(inner.max.x + STRIP, inner.max.y),
        )
    } else {
        Rect::from_min_max(
            pos2(inner.min.x, inner.max.y),
            pos2(inner.max.x, inner.max.y + STRIP),
        )
    }
}

/// The strip's span across the bar and its full span along the content.
fn spans(strip: Rect, along: usize) -> (Rangef, Rangef) {
    let across = 1 - along;
    (
        Rangef::new(strip.min[across], strip.max[across]),
        Rangef::new(strip.min[along], strip.max[along]),
    )
}

/// The handle's span along the strip: its length is the share of the content
/// on show, at least [`MIN_LENGTH`], and it travels between the strip's ends
/// less [`END_INSET`].
pub(super) fn thumb_span(along: Rangef, viewport: f32, content: f32, offset: f32) -> Rangef {
    let track = Rangef::new(along.min + END_INSET, along.max - END_INSET);
    let len = track.span();
    let length = (len * viewport / content).max(MIN_LENGTH).min(len);

    let travel = len - length;
    let max_offset = content - viewport;
    let share = if max_offset > 0.0 {
        (offset / max_offset).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let start = travel.mul_add(share, track.min);
    Rangef::new(start, start + length)
}

/// The span across the strip of a handle or a groove `width` wide, one pixel
/// in from the strip's near edge.
pub(super) fn cross_span(strip: Rangef, width: f32) -> Rangef {
    let centre = strip.min + EDGE + (STRIP - EDGE) / 2.0;
    Rangef::new(centre - width / 2.0, centre + width / 2.0)
}

fn rect(along: usize, along_span: Rangef, across_span: Rangef) -> Rect {
    if along == 1 {
        Rect::from_x_y_ranges(across_span, along_span)
    } else {
        Rect::from_x_y_ranges(along_span, across_span)
    }
}

/// Half of the width, as egui rounds a corner: a pill.
fn pill(width: f32) -> CornerRadius {
    CornerRadius::same(if width > 6.0 { 4 } else { 2 })
}

/// Whether the content behind the bar has the focus, as Darkly reads its
/// scrolled parent.
fn is_focused<R>(ctx: &Context, output: &ScrollAreaOutput<R>) -> bool {
    let Some(focus) = ctx.memory(egui::Memory::focused) else {
        return false;
    };

    focus == output.id
        || ctx
            .read_response(focus)
            .is_some_and(|response| output.inner_rect.contains(response.rect.center()))
}
