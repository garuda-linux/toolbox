use egui::emath::GuiRounding as _;
use egui::layers::ShapeIdx;
use egui::{
    Color32, CornerRadius, Frame, InnerResponse, Margin, Rect, Response, Sense, Shape, Stroke, Ui,
    UiBuilder,
};

use super::list::memory;
use super::press::{Pointer, hover_amount};
use crate::theme::{CARD_RADIUS, RADIUS};
use crate::tokens;

pub(super) const HOVER_INSET: f32 = 4.0;
pub(super) const DIVIDER: f32 = 1.0;

pub fn card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let tokens = tokens::current(ui.ctx());
    Frame::new()
        .fill(tokens.card)
        .stroke(light_rim(ui, tokens.outline))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            body(ui)
        })
}

/// A [`card`] with the card padding round its body.
pub fn padded_card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let padding = tokens::current(ui.ctx()).sizes.card_padding;
    card(ui, |ui| {
        Frame::new()
            .inner_margin(padding)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                body(ui)
            })
            .inner
    })
}

/// A 1 px rim of `colour` in light themes only, where white on near
/// white needs one. Dark themes set a surface off by its fill alone.
pub(super) fn light_rim(ui: &Ui, colour: Color32) -> Stroke {
    if ui.visuals().dark_mode {
        Stroke::NONE
    } else {
        Stroke::new(1.0, colour)
    }
}

pub fn card_row<R>(ui: &mut Ui, first: bool, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    card_row_with(ui, first, true, body)
}

/// A [`card_row`] that shows the hover fill only when `activatable`, as
/// a row a click acts on. A row of facts or of its own buttons alone
/// stays still under the pointer.
pub fn card_row_with<R>(
    ui: &mut Ui,
    first: bool,
    activatable: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    divider(ui, first);
    let backdrop = ui.painter().add(Shape::Noop);
    let inner = ui.scope_builder(UiBuilder::new().sense(Sense::hover()), |ui| {
        let response = ui.response();
        let dragging = ui.ctx().dragged_id().is_some();
        let lit = shows_hover(response.hovered(), response.contains_pointer(), dragging);
        mark_raised(ui, activatable && lit);
        row_frame(ui, body)
    });

    if activatable {
        paint_hover(ui, backdrop, &inner.response);
    }

    inner
}

/// The line above every row but a card's first, on a whole pixel so the
/// fill below it can keep an exact gap. The line takes its own pixel, so
/// the row below keeps its whole padding clear of it.
pub(super) fn divider(ui: &mut Ui, first: bool) {
    if first {
        return;
    }

    let tokens = tokens::current(ui.ctx());
    let padding = tokens.sizes.row_padding;
    let left = ui.max_rect().left() + padding.x;
    let right = ui.max_rect().right() - padding.x;
    let y = ui.painter().round_to_pixel_center(ui.cursor().top());
    ui.painter()
        .hline(left..=right, y, Stroke::new(DIVIDER, tokens.outline));
    ui.add_space(y + DIVIDER / 2.0 - ui.cursor().top());
}

pub(super) fn row_frame<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    let sizes = tokens::current(ui.ctx()).sizes;
    Frame::new()
        .inner_margin(Margin::from(sizes.row_padding))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = sizes.tight_gap;
            body(ui)
        })
        .inner
}

/// Painted once the row's size is known (Rerun's list items). Covered by its own
/// button the row still counts as hovered, but not while something else is dragged across it.
pub(super) fn paint_hover(ui: &Ui, backdrop: ShapeIdx, response: &Response) {
    let dragging = ui.ctx().dragged_id().is_some();
    let pointer = if shows_hover(response.hovered(), response.contains_pointer(), dragging) {
        Pointer::Hovered
    } else {
        Pointer::Rest
    };

    let amount = hover_amount(ui, response, pointer);
    let rect = fill_rect(response.rect, ui.pixels_per_point());
    let tokens = tokens::current(ui.ctx());
    let fill = Shape::rect_filled(
        rect,
        CornerRadius::same(RADIUS),
        tokens.field.gamma_multiply(amount),
    );

    ui.painter().set(backdrop, fill);
}

/// A row under the pointer shows its fill, also while its own button is
/// pressed, which takes the hover from the row. While something is
/// dragged only the dragged widget counts as hovered.
pub(super) const fn shows_hover(hovered: bool, contains_pointer: bool, dragging: bool) -> bool {
    hovered || (contains_pointer && !dragging)
}

/// Where the fill and focus ring of `row` go: [`HOVER_INSET`] inside the
/// card's edges and as far from the divider above as from the next one
/// below. The divider above ends where the row starts, the next one
/// takes the pixel the next row starts on.
pub(super) fn fill_rect(row: Rect, pixels_per_point: f32) -> Rect {
    let top = row.top();
    let bottom = row.bottom().round_to_pixel_center(pixels_per_point) - DIVIDER / 2.0;
    Rect::from_x_y_ranges(
        row.left() + HOVER_INSET..=row.right() - HOVER_INSET,
        top + HOVER_INSET..=bottom - HOVER_INSET,
    )
}

pub(super) fn mark_raised(ui: &Ui, raised: bool) {
    memory::write(ui.ctx(), ui.scope_id(), |row| row.raised = Some(raised));
}

/// Whether `ui` or a Ui around it sits on a row's hover fill, where a
/// Soft button at rest has the fill's own colour.
pub(super) fn on_raised(ui: &Ui) -> bool {
    ui.stack()
        .iter()
        .any(|frame| memory::get(ui, frame.scope_id).raised == Some(true))
}

/// Whether `ui` sits in a card's row, which paints the one fill under
/// the pointer for all it holds.
pub(super) fn in_card_row(ui: &Ui) -> bool {
    ui.stack()
        .iter()
        .any(|frame| memory::get(ui, frame.scope_id).raised.is_some())
}
