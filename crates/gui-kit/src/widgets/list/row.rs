//! One row of a list: its divider, its own id and focus, the name screen
//! readers hear, the hover fill and the keyboard focus line.

use egui::layers::ShapeIdx;
use egui::style::ScrollAnimation;
use egui::{Frame, Id, Margin, Rect, Response, Role, Shape, Ui, UiBuilder, WidgetInfo, vec2};

use super::super::card::{divider, mark_raised, paint_hover, row_frame, shows_hover};
use super::super::focus::item_focus_line;
use super::super::layout::{SETTLED, moved, remeasured};
use super::super::roving;
use super::{ListItem, RowState, memory};
use crate::{motion, tokens};

/// Where a row goes and what the keys asked of it.
#[derive(Clone, Copy, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "a row keeps each key and state as a flag of its own"
)]
pub struct Place {
    /// The list's own id, whose kept offsets a new height makes stale.
    pub list: Id,
    pub id: Id,
    pub index: usize,
    /// Whether the row is the list's Tab stop, the only focusable row.
    pub focusable: bool,
    pub fixed: Option<f32>,
    pub toggle: bool,
    pub menu: bool,
    pub selected: bool,
    pub keyed: bool,
}

/// Lays out one row and returns its height with its response.
pub fn show<T>(
    ui: &mut Ui,
    place: Place,
    item: T,
    body: &mut impl FnMut(&mut Ui, T, &RowState),
) -> (f32, Response)
where
    T: ListItem,
{
    // The body takes the item, the screen reader's name is set after it.
    let name = item.name().to_owned();
    let start = ui.cursor().top();
    divider(ui, place.index == 0);
    let backdrop = ui.painter().add(Shape::Noop);
    if place.toggle {
        motion::mark_keyboard(ui.ctx(), place.id);
    }

    // An id of its own keeps what is inside with the item, wherever the
    // row moves in the list.
    let mut builder = UiBuilder::new()
        .scope_id(place.id)
        .sense(roving::sense(place.focusable));

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
            row: place.id,
            toggle: place.toggle || whole.clicked(),
            response: whole,
            menu: place.menu,
            selected: place.selected,
            keyed: place.keyed,
        };

        let dragging = ui.ctx().dragged_id().is_some();
        let response = &state.response;
        let lit = shows_hover(response.hovered(), response.contains_pointer(), dragging);
        mark_raised(ui, lit);

        match place.fixed {
            Some(height) => centred_frame(ui, height, place, |ui| body(ui, item, &state)),
            None => row_frame(ui, |ui| body(ui, item, &state)),
        }
    });

    let response = inner.response;
    response.widget_info(|| WidgetInfo::labeled(Role::ListItem, ui.is_enabled(), &name));

    focus(ui, &response, place.selected);
    paint_fill(ui, backdrop, &response, place.selected);

    // The divider above counts, so the rows below are placed past it.
    let height = response.rect.bottom() - start;
    if place.fixed.is_none() {
        remember(ui, place, height);
    }

    (height, response)
}

/// A row of a fixed height around `body`, its content centred in the
/// slot by the height it took last frame, so a row with fewer lines than
/// the list's tallest leaves no gap under its text.
fn centred_frame(ui: &mut Ui, height: f32, place: Place, body: impl FnOnce(&mut Ui)) {
    let sizes = tokens::current(ui.ctx()).sizes;
    let known = memory::get(ui, place.id).content;
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
            if known.is_none_or(|was| moved(was, content)) {
                memory::update(ui, (place.list, place.id), |row| {
                    row.content = Some(content);
                });
                remeasured(ui, known.is_none(), "fixed row content");
            }
        });
}

/// A selected row fills its whole cell with the selection colour, lighter
/// under the pointer, as Darkly's item view draws a selected item. Other
/// rows keep the hover fill.
fn paint_fill(ui: &Ui, backdrop: ShapeIdx, response: &Response, selected: bool) {
    if !selected {
        paint_hover(ui, backdrop, response);
        return;
    }

    let dragging = ui.ctx().dragged_id().is_some();
    let lit = shows_hover(response.hovered(), response.contains_pointer(), dragging);
    let tokens = tokens::current(ui.ctx());
    let fill = if lit {
        tokens.tint_hover
    } else {
        tokens.selection
    };

    ui.painter()
        .set(backdrop, Shape::rect_filled(response.rect, 0.0, fill));
}

/// A click on the row's empty part focuses it (Rerun's list items), and
/// a focused row scrolls into view.
fn focus(ui: &Ui, response: &Response, selected: bool) {
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

    item_focus_line(ui, response, response.rect, selected);
}

/// The height row `id` took when last laid out.
pub(super) fn remembered(ui: &Ui, id: Id) -> Option<f32> {
    memory::get(ui, id).height
}

fn remember(ui: &Ui, place: Place, height: f32) {
    let id = place.id;
    if remembered(ui, id).is_some_and(|known| (known - height).abs() < SETTLED) {
        return;
    }

    memory::update(ui, (place.list, id), |row| row.height = Some(height));

    // The rows below moved, so the next frame places them anew.
    heights_changed(ui, place.list);
    ui.ctx().request_repaint();
}

/// Counts the measured heights of `list` that changed, so the list
/// builds its offsets again only then.
pub(super) fn heights_seen(ui: &Ui, list: Id) -> u64 {
    ui.data(|data| data.get_temp::<u64>(heights_id(list)))
        .unwrap_or_default()
}

/// The kept offsets of `list` no longer fit its rows.
pub(super) fn heights_changed(ui: &Ui, list: Id) {
    ui.data_mut(|data| {
        let seen = data.get_temp_mut_or_default::<u64>(heights_id(list));
        *seen = seen.wrapping_add(1);
    });
}

fn heights_id(list: Id) -> Id {
    list.with("heights-changed")
}
