//! Options side by side in one pill, the chosen one raised, as views of
//! one page (System and User units). A radio group to a screen reader:
//! one Tab stop, the arrow keys move the choice.

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, EventFilter, Galley, Id, Key, Modifiers, Rect, Response, Role, Sense,
    StrokeKind, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};

use super::card::light_rim;
use super::press::Pointer;
use super::{focus_ring, line};
use crate::fonts::{BODY, Weight};
use crate::motion;
use crate::theme::{PILL_HEIGHT, PILL_RADIUS};
use crate::tokens::{self, Tokens};

/// Between the pill's edge and a segment.
const INSET: f32 = 3.0;
const PADDING_X: f32 = 16.0;
/// Round ends of a segment, half its height inside the inset.
const SEGMENT_RADIUS: u8 = 14;

/// How wide [`segmented`] draws `labels`, for a row that places it before
/// drawing.
pub fn segmented_width(ui: &Ui, labels: &[&str]) -> f32 {
    let segments: f32 = labels
        .iter()
        .map(|label| segment_width(&label_galley(ui, label)))
        .sum();

    INSET.mul_add(2.0, segments)
}

fn label_galley(ui: &Ui, label: &str) -> Arc<Galley> {
    line(ui, label, Weight::SemiBold, BODY, Color32::PLACEHOLDER)
}

fn segment_width(galley: &Galley) -> f32 {
    PADDING_X.mul_add(2.0, galley.size().x)
}

/// One segment per option, the one equal to `current` raised, in a group
/// called `name`. Returns the option clicked or moved to this frame.
pub fn segmented<T>(ui: &mut Ui, name: &str, options: &[(T, &str)], current: T) -> Option<T>
where
    T: PartialEq + Copy,
{
    let galleys: Vec<_> = options
        .iter()
        .map(|(_, label)| label_galley(ui, label))
        .collect();

    let widths: Vec<f32> = galleys.iter().map(|galley| segment_width(galley)).collect();

    // Taken here, so a wrapping row moves the whole pill to the next line.
    let total = INSET.mul_add(2.0, widths.iter().sum::<f32>());
    let height = PILL_HEIGHT;
    let (outer, _) = ui.allocate_exact_size(vec2(total, height), Sense::hover());
    let builder = UiBuilder::new()
        .max_rect(outer)
        .id_salt(("segmented", name))
        .accessibility_role(Role::RadioGroup)
        .accessibility_label(name);

    let group = ui.new_child(builder);
    segments(&group, outer, (galleys, widths), options, current)
}

fn segments<T>(
    ui: &Ui,
    outer: Rect,
    (galleys, widths): (Vec<Arc<Galley>>, Vec<f32>),
    options: &[(T, &str)],
    current: T,
) -> Option<T>
where
    T: PartialEq + Copy,
{
    let tokens = tokens::current(ui.ctx());
    ui.painter().rect_filled(outer, PILL_RADIUS, tokens.field);

    let mut left = outer.left() + INSET;
    let rects: Vec<Rect> = widths
        .iter()
        .map(|width| {
            let rect = Rect::from_min_size(
                pos2(left, outer.top() + INSET),
                vec2(*width, INSET.mul_add(-2.0, outer.height())),
            );

            left += width;
            rect
        })
        .collect();

    let (shown, mut picked) = keyed(ui, options, current);
    if let Some(rect) = shown.and_then(|index| rects.get(index)) {
        pill(ui, *rect, picked.is_some());
    }

    for (index, ((value, label), (galley, rect))) in options
        .iter()
        .zip(galleys.into_iter().zip(rects))
        .enumerate()
    {
        let chosen = Some(index) == shown;
        let response = segment(ui, segment_id(ui, label), rect, label, chosen);
        let ink = if chosen || response.hovered() {
            tokens.text
        } else {
            tokens.secondary
        };

        ui.painter()
            .galley(rect.center() - galley.size() / 2.0, galley, ink);

        if response.has_focus() {
            lock_arrows(ui, &response);
        }

        if response.clicked() {
            picked = Some(*value);
        }
    }

    picked
}

/// The segment shown as chosen and the option an arrow key picked. A key
/// on the focused segment moves the choice and the focus before the
/// segments are drawn, so the new one can take focus in this frame.
fn keyed<T>(ui: &Ui, options: &[(T, &str)], current: T) -> (Option<usize>, Option<T>)
where
    T: PartialEq + Copy,
{
    let shown = options.iter().position(|(value, _)| *value == current);
    let focused = shown
        .and_then(|index| options.get(index))
        .is_some_and(|(_, label)| ui.memory(|memory| memory.has_focus(segment_id(ui, label))));

    let next = shown
        .filter(|_| focused)
        .and_then(|index| moved(ui, index, options.len()))
        .and_then(|next| Some((next, options.get(next)?)));

    match next {
        Some((next, (value, label))) => {
            ui.memory_mut(|memory| memory.request_focus(segment_id(ui, label)));
            (Some(next), Some(*value))
        }
        None => (shown, None),
    }
}

/// The raised pill under the chosen segment, at `rect`. A click slides
/// it there, a key moves it at once, as keys never wait for motion.
fn pill(ui: &Ui, rect: Rect, by_key: bool) {
    let ctx = ui.ctx();
    let time = if by_key {
        0.0
    } else {
        motion::duration(ctx, motion::SEGMENT_SLIDE)
    };

    let id = ui.scope_id().with("pill");
    let left = ctx.animate_value_with_time(id.with("left"), rect.left(), time);
    let width = ctx.animate_value_with_time(id.with("width"), rect.width(), time);
    let at = Rect::from_min_size(pos2(left, rect.top()), vec2(width, rect.height()));

    let tokens = tokens::current(ctx);

    // White on pale grey needs a rim to show which one is chosen.
    let rim = light_rim(ui, tokens.rim);

    ui.painter().rect(
        at,
        CornerRadius::same(SEGMENT_RADIUS),
        tokens.raised,
        rim,
        StrokeKind::Inside,
    );
}

fn segment_id(ui: &Ui, label: &str) -> Id {
    ui.scope_id().with(("segment", label))
}

/// Only the chosen segment takes focus, so Tab passes the group in one
/// step. A click still picks the others.
fn segment(ui: &Ui, id: Id, rect: Rect, label: &str, chosen: bool) -> Response {
    let tokens = tokens::current(ui.ctx());
    let sense = if chosen { Sense::click() } else { Sense::CLICK };
    let response = ui.interact(rect, id, sense);
    response
        .widget_info(|| WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), chosen, label));

    if let Some(fill) = segment_fill(&tokens, chosen, Pointer::of(&response)) {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(SEGMENT_RADIUS), fill);
    }

    focus_ring(ui, &response, SEGMENT_RADIUS);
    response
}

/// A segment's own fill: only another one than the chosen, while pressed.
/// The chosen one sits on the sliding [`pill`], and a hover shows in the
/// text alone.
pub(super) const fn segment_fill(
    tokens: &Tokens,
    chosen: bool,
    pointer: Pointer,
) -> Option<Color32> {
    match (chosen, pointer) {
        (false, Pointer::Pressed) => Some(tokens.pressed),
        (true, _) | (false, Pointer::Rest | Pointer::Hovered) => None,
    }
}

/// Keeps the arrow keys for the group while a segment has focus, instead
/// of egui moving the focus with them.
fn lock_arrows(ui: &Ui, response: &Response) {
    let filter = EventFilter {
        horizontal_arrows: true,
        vertical_arrows: true,
        ..EventFilter::default()
    };

    ui.memory_mut(|memory| memory.set_focus_lock_filter(response.id, filter));
}

/// The segment an arrow key moves to from `index`, wrapping at the ends
/// as radio groups do.
fn moved(ui: &Ui, index: usize, count: usize) -> Option<usize> {
    let (back, forward) = ui.input_mut(|input| {
        let back = input.consume_key(Modifiers::NONE, Key::ArrowLeft)
            | input.consume_key(Modifiers::NONE, Key::ArrowUp);

        let forward = input.consume_key(Modifiers::NONE, Key::ArrowRight)
            | input.consume_key(Modifiers::NONE, Key::ArrowDown);

        (back, forward)
    });

    let next = if forward {
        index.saturating_add(1).checked_rem(count)
    } else if back {
        index.checked_sub(1).or_else(|| count.checked_sub(1))
    } else {
        None
    };

    next.filter(|next| *next != index)
}
