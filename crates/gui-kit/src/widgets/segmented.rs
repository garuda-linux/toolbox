//! Options side by side as rounded tabs, the chosen one a pill raised above
//! the others, as views of one page (System and User units). The tabs join
//! each other.
//! A radio group to a screen reader: one Tab stop, the arrow keys move the
//! choice.

use std::sync::Arc;

use egui::{
    AsIdSalt, Color32, CornerRadius, EventFilter, Galley, Id, Key, Modifiers, Rect, Response, Role,
    Sense, Shadow, Shape, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};

use super::buttons::look::BLUR_PER_RADIUS;
use super::focus::focus_from_keyboard;
use super::press::Pointer;
use super::{Choice, line, push_height};
use crate::fonts::{BODY, Weight};
use crate::theme::{RADIUS, TAB_RADIUS};
use crate::tokens::{self, Tokens};

/// Breeze's `TabBar_TabMinWidth` (breezemetrics.h:131).
const TAB_MIN_WIDTH: f32 = 80.0;
/// Breeze's `TabBar_TabMarginWidth`, which Qt's `PM_TabBarTabHSpace` takes
/// on each side of the text (breezemetrics.h:130, breezestyle.cpp:821). The
/// chosen pill sits inside the tab by `PILL_INSET`, so the tab adds that
/// inset to keep the text margin inside the pill.
const TAB_MARGIN_X: f32 = 8.0 + PILL_INSET;
/// Breeze's `TabBar_TabMarginHeight`, which `PM_TabBarTabVSpace` takes
/// above and below the text (breezemetrics.h:129, breezestyle.cpp:823).
const TAB_MARGIN_Y: f32 = 4.0;
/// Breeze's `TabBar_TabOverlap`: each tab runs this far under the next
/// (breezemetrics.h:135, breezestyle.cpp:817).
const TAB_OVERLAP: f32 = 1.0;
/// Darkly's pill of the chosen tab sits this far inside the tab rect
/// (darklystyle.cpp:7354-7355).
pub(super) const PILL_INSET: f32 = 4.0;
/// Above and below, the pill keeps the inset of a field's or button's face,
/// so a tab is as tall as the controls beside it in a toolbar.
const PILL_INSET_Y: f32 = 2.0;

/// How far every tab's pill sits inside its rect, for a row that places
/// the tabs by what they paint.
pub const fn tab_pill_inset() -> f32 {
    PILL_INSET
}

/// How wide [`segmented`] draws `labels`, for a row that places it before
/// drawing.
pub fn segmented_width(ui: &Ui, labels: &[&str]) -> f32 {
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| segment_width(&label_galley(ui, label)))
        .collect();

    total_width(&widths)
}

/// The width of tabs side by side, each one over the next by the overlap.
fn total_width(widths: &[f32]) -> f32 {
    let between = widths.len().saturating_sub(1);
    let overlap = f32::from(u16::try_from(between).unwrap_or(u16::MAX)) * TAB_OVERLAP;
    widths.iter().sum::<f32>() - overlap
}

fn label_galley(ui: &Ui, label: &str) -> Arc<Galley> {
    line(ui, label, Weight::SemiBold, BODY, Color32::PLACEHOLDER)
}

fn segment_width(galley: &Galley) -> f32 {
    TAB_MARGIN_X
        .mul_add(2.0, galley.size().x)
        .max(TAB_MIN_WIDTH)
}

/// Tabs are a push button high, not Breeze's `TabBar_TabMinHeight` of 30
/// (breezemetrics.h:132).
fn tab_height(ui: &Ui, galleys: &[Arc<Galley>]) -> f32 {
    galleys
        .iter()
        .map(|galley| TAB_MARGIN_Y.mul_add(2.0, galley.size().y))
        .fold(push_height(ui), f32::max)
}

/// One tab per option, the one equal to `current` chosen, in a group
/// called `name`. Returns the option clicked or moved to this frame.
pub fn segmented<T>(
    ui: &mut Ui,
    salt: impl AsIdSalt,
    name: &str,
    options: &[(T, &str)],
    current: T,
) -> Choice<T>
where
    T: PartialEq + Copy,
{
    let galleys: Vec<_> = options
        .iter()
        .map(|(_, label)| label_galley(ui, label))
        .collect();

    let widths: Vec<f32> = galleys.iter().map(|galley| segment_width(galley)).collect();

    // Taken here, so a wrapping row moves the whole group to the next line.
    let total = total_width(&widths);
    let (outer, response) =
        ui.allocate_exact_size(vec2(total, tab_height(ui, &galleys)), Sense::hover());

    let builder = UiBuilder::new()
        .max_rect(outer)
        .id_salt(("segmented", salt))
        .accessibility_role(Role::RadioGroup)
        .accessibility_label(name);

    let group = ui.new_child(builder);
    let value = segments(&group, outer, (galleys, widths), options, current);
    Choice { response, value }
}

/// The tabs end to end across `outer`, each joined to the next by the overlap.
fn tab_rects(outer: Rect, widths: &[f32]) -> Vec<Rect> {
    let mut left = outer.left();
    widths
        .iter()
        .map(|width| {
            let rect = Rect::from_min_size(pos2(left, outer.top()), vec2(*width, outer.height()));
            left += width - TAB_OVERLAP;
            rect
        })
        .collect()
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
    let rects = tab_rects(outer, &widths);

    let (shown, mut picked) = keyed(ui, options, current);
    let responses: Vec<Response> = options
        .iter()
        .zip(&rects)
        .enumerate()
        .map(|(index, ((_, label), rect))| {
            segment(
                ui,
                segment_id(ui, label),
                *rect,
                label,
                Some(index) == shown,
            )
        })
        .collect();

    // Breeze paints the chosen tab last, over the tabs it overlaps.
    let order = (0..options.len())
        .filter(|index| Some(*index) != shown)
        .chain(shown);

    for index in order {
        let (Some(rect), Some(galley), Some(response), Some((value, _))) = (
            rects.get(index),
            galleys.get(index),
            responses.get(index),
            options.get(index),
        ) else {
            continue;
        };

        let chosen = Some(index) == shown;
        let corners = corners(chosen);
        let focused = focus_from_keyboard(ui, response) && ui.is_enabled();
        paint_tab(
            ui,
            &tokens,
            (*rect, corners),
            (chosen, Pointer::of(response)),
            focused,
        );

        // The label is laid out once in the text colour, so a focused tab
        // paints its own colour over it.
        let ink = if focused {
            tokens.on_focus
        } else {
            tokens.text
        };

        ui.painter()
            .galley(rect.center() - galley.size() / 2.0, Arc::clone(galley), ink);

        if response.has_focus() {
            lock_arrows(ui, response);
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

fn segment_id(ui: &Ui, label: &str) -> Id {
    ui.scope_id().with(("segment", label))
}

/// Only the chosen segment takes focus, so Tab passes the group in one
/// step. A click still picks the others.
fn segment(ui: &Ui, id: Id, rect: Rect, label: &str, chosen: bool) -> Response {
    let sense = if chosen { Sense::click() } else { Sense::CLICK };
    let response = ui.interact(rect, id, sense);
    response
        .widget_info(|| WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), chosen, label));

    response
}

/// The chosen tab is a pill inset from its rect, with Darkly's shadow and
/// a 1 px outline. An unchosen one has no fill of its own unless the pointer or
/// the keyboard is on it, and then fills the same inset pill, so it is
/// never larger than the choice beside it.
fn paint_tab(
    ui: &Ui,
    tokens: &Tokens,
    (rect, corners): (Rect, CornerRadius),
    (is_chosen, pointer): (bool, Pointer),
    focused: bool,
) {
    let body = rect.shrink2(vec2(PILL_INSET, PILL_INSET_Y));

    let shapes = ui.painter();
    if is_chosen {
        shapes.add(pill_shadow(tokens, body, corners));
    }

    shapes.rect_filled(body, corners, tab_fill(tokens, is_chosen, pointer, focused));
    if is_chosen {
        // The fill alone sits near the window colour on flat palettes.
        shapes.rect_stroke(
            body,
            corners,
            Stroke::new(1.0, tokens.card_outline),
            StrokeKind::Inside,
        );
    }
}

/// Darkly's shadow under the chosen pill, one pixel down. Qt blurs it 6
/// and egui draws twice Qt's blur (darklystyle.cpp:7355-7360,
/// darklyhelper.cpp:1791-1808).
fn pill_shadow(tokens: &Tokens, reach: Rect, corners: CornerRadius) -> Shape {
    let shadow = Shadow {
        offset: [0, 1],
        blur: 6 * BLUR_PER_RADIUS,
        spread: 0,
        color: tokens.control_shadow,
    };

    Shape::from(shadow.as_shape(reach, corners))
}

/// Every corner of a tab is rounded: the chosen one as the pill of
/// `RADIUS`, the others at `TAB_RADIUS`.
pub(super) const fn corners(chosen: bool) -> CornerRadius {
    CornerRadius::same(if chosen { RADIUS } else { TAB_RADIUS })
}

/// The fill of a tab. The chosen one is the push-button colour, the fill
/// that stands apart from the window on a flat theme. An unchosen one has
/// no fill at rest and the item hover under the pointer.
pub(super) const fn segment_fill(tokens: &Tokens, chosen: bool, pointer: Pointer) -> Color32 {
    if chosen {
        tokens.chosen_tab
    } else if pointer.is_under() {
        tokens.item_hover
    } else {
        Color32::TRANSPARENT
    }
}

/// A tab with keyboard focus fills with the focus colour, as a focused
/// button does. Otherwise it is [`segment_fill`].
pub(super) const fn tab_fill(
    tokens: &Tokens,
    chosen: bool,
    pointer: Pointer,
    focused: bool,
) -> Color32 {
    if focused {
        tokens.focus_fill
    } else {
        segment_fill(tokens, chosen, pointer)
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
