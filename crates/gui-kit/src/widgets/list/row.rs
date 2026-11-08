//! One row of a list: its divider, its own id and focus, the name screen
//! readers hear, the hover fill and the focus ring.

use egui::style::ScrollAnimation;
use egui::{
    EventFilter, Frame, Id, Margin, Rect, Response, Role, Sense, Shape, Ui, UiBuilder, WidgetInfo,
    vec2,
};

use super::super::card::{divider, fill_rect, mark_raised, paint_hover, row_frame, shows_hover};
use super::super::focus::ring_around;
use super::super::layout::{SETTLED, remember_width};
use super::{ListItem, RowState};
use crate::theme::RADIUS;
use crate::{motion, tokens};

/// Where a row goes and what the keys asked of it.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub id: Id,
    pub index: usize,
    pub fixed: Option<f32>,
    pub toggle: bool,
    pub menu: bool,
}

pub fn show<T>(
    ui: &mut Ui,
    place: Place,
    item: &T,
    body: &mut impl FnMut(&mut Ui, &T, &RowState),
) -> f32
where
    T: ListItem,
{
    let start = ui.cursor().top();
    divider(ui, place.index == 0);
    let backdrop = ui.painter().add(Shape::Noop);
    if place.toggle {
        motion::mark_keyboard(ui.ctx(), place.id);
    }

    // An id of its own keeps what is inside with the item, wherever the
    // row moves in the list.
    let mut builder = UiBuilder::new().scope_id(place.id).sense(Sense::click());
    if let Some(height) = place.fixed {
        let slot = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), height));
        builder = builder.max_rect(slot);
    }

    let inner = ui.scope_builder(builder, |ui| {
        if let Some(height) = place.fixed {
            ui.set_min_height(height);
        }

        // A click on the row's empty part does what Space does, so a
        // switch's target is its whole row.
        let whole = ui.response();
        let state = RowState {
            index: place.index,
            toggle: place.toggle || whole.clicked(),
            response: whole,
            menu: place.menu,
        };

        let dragging = ui.ctx().dragged_id().is_some();
        let response = &state.response;
        let lit = shows_hover(response.hovered(), response.contains_pointer(), dragging);
        mark_raised(ui, lit || response.has_focus());

        match place.fixed {
            Some(height) => centred_frame(ui, height, place.id, |ui| body(ui, item, &state)),
            None => row_frame(ui, |ui| body(ui, item, &state)),
        }
    });

    let response = inner.response;
    response.widget_info(|| WidgetInfo::labeled(Role::ListItem, ui.is_enabled(), item.name()));

    focus(ui, &response);
    paint_hover(ui, backdrop, &response);

    // The divider above counts, so the rows below are placed past it.
    let height = response.rect.bottom() - start;
    if place.fixed.is_none() {
        remember(ui, place.id, height);
    }

    height
}

/// A row of a fixed height around `body`, its content centred in the
/// slot by the height it took last frame, so a row with fewer lines than
/// the list's tallest leaves no gap under its text.
fn centred_frame(ui: &mut Ui, height: f32, id: Id, body: impl FnOnce(&mut Ui)) {
    let sizes = tokens::current(ui.ctx()).sizes;
    let content_id = id.with("content");
    let known = ui.data(|data| data.get_temp::<f32>(content_id));
    let top = known.map_or(sizes.row_padding.y, |content| {
        ((height - content) / 2.0).max(0.0)
    });

    Frame::new()
        .inner_margin(Margin::from(vec2(sizes.row_padding.x, 0.0)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add_space(top);
            let drawn = ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = sizes.tight_gap;
                body(ui);
            });

            let content = drawn.response.rect.height();
            remember_width(ui, content_id, known, content, "fixed row content");
        });
}

/// A click on the row's empty part focuses it (Rerun's list items), and
/// a focused row scrolls into view and keeps Up and Down for the list.
fn focus(ui: &Ui, response: &Response) {
    if response.clicked() {
        response.request_focus();
    }

    // Focus a key moved scrolls at once, so a held arrow never waits for
    // the scroll to catch up. A click's focus keeps the style's scroll.
    if response.gained_focus() {
        let from_pointer =
            ui.input(|input| input.pointer.any_down() || input.pointer.any_released());

        if from_pointer {
            response.scroll_to_me(None);
        } else {
            response.scroll_to_me_animation(None, ScrollAnimation::none());
        }
    }

    if response.has_focus() {
        let filter = EventFilter {
            vertical_arrows: true,
            ..EventFilter::default()
        };

        ui.memory_mut(|memory| memory.set_focus_lock_filter(response.id, filter));
        let ring = fill_rect(response.rect, ui.pixels_per_point());
        ring_around(ui, ring, RADIUS);
    }
}

/// The height row `id` took when last laid out.
pub fn remembered(ui: &Ui, id: Id) -> Option<f32> {
    ui.data(|data| data.get_temp::<f32>(height_id(id)))
}

fn remember(ui: &Ui, id: Id, height: f32) {
    if remembered(ui, id).is_some_and(|known| (known - height).abs() < SETTLED) {
        return;
    }

    ui.data_mut(|data| data.insert_temp(height_id(id), height));

    // The rows below moved, so the next frame places them anew.
    ui.ctx().request_repaint();
}

fn height_id(id: Id) -> Id {
    id.with("height")
}
