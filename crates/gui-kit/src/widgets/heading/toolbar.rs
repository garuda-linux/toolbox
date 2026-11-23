//! The toolbar row's search and sort, drawn in the room a page header leaves
//! beside its title.

use egui::{Align, Layout, Rect, Ui, UiBuilder};

use super::super::field::INSET;
use super::super::select::pill::FRAME;
use super::SEARCH_LEAST;
use crate::tokens;

/// A page's search (`left`) and sort (`right`) in the toolbar row of the
/// [`page_header`] just drawn.
///
/// They sit at the row's right end with the sort last, as Discover and
/// Dolphin place them. Where the title leaves too little room
/// beside them, they take a row of their own under it, search at the left
/// and sort at the right. Both draw from `page`, which they take in turn.
pub fn page_toolbar<P>(
    ui: &mut Ui,
    page: &mut P,
    left: impl FnOnce(&mut P, &mut Ui),
    right: impl FnOnce(&mut P, &mut Ui),
) {
    let room = ui.ctx().data_mut(|data| {
        let room = data.get_temp::<Rect>(toolbar_room_id());
        data.remove::<Rect>(toolbar_room_id());
        room
    });

    let width_id = ui.scope_id().with("page_toolbar_width");
    let needed = ui
        .ctx()
        .data(|data| data.get_temp::<f32>(width_id))
        .unwrap_or_else(|| tokens::current(ui.ctx()).sizes.search_width);

    if let Some(room) = room.filter(|room| room.width() >= needed) {
        let drawn = ui.scope_builder(
            UiBuilder::new()
                .max_rect(room)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| _ = sort_then_search(ui, page, left, right),
        );

        let width = drawn.response.rect.width();
        ui.ctx().data_mut(|data| data.insert_temp(width_id, width));
        return;
    }

    ui.add_space(tokens::current(ui.ctx()).sizes.card_gap);

    // The sort is drawn first from the right, as on the room path, so the search
    // fills what it leaves. The stored width is what the title must leave, not
    // the drawn row, which is full width and would keep the toolbar on its own row.
    let sort = ui
        .horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                sort_then_search(ui, page, left, right)
            })
            .inner
        })
        .inner;

    ui.ctx()
        .data_mut(|data| data.insert_temp(width_id, sort + SEARCH_LEAST));
}

/// Draws the sort from the right, then the search left of it, and returns
/// the room the sort took with its gap, zero without one.
///
/// The field paints its outline `INSET` inside its rect and the sort's
/// caption starts at its own edge, so the search moves that much closer for
/// the group gap to show between what both paint.
fn sort_then_search<P>(
    ui: &mut Ui,
    page: &mut P,
    left: impl FnOnce(&mut P, &mut Ui),
    right: impl FnOnce(&mut P, &mut Ui),
) -> f32 {
    let start = ui.cursor().max.x;
    right(page, ui);
    let mut sort = start - ui.cursor().max.x;
    if sort > 0.0 {
        ui.add_space(-INSET);
        sort -= INSET;
    }

    left(page, ui);
    sort
}

/// How far inside the toolbar's right end its last control paints.
///
/// That is the sort's pill, or the search's outline where a page has no
/// sort. The lesser of both, so the gap after it never shrinks below the
/// group gap.
pub const fn toolbar_end_inset() -> f32 {
    FRAME.min(INSET)
}

/// The room between the toolbar's rects and the trailing controls' rects.
///
/// With an `inset` both paint inside them, the room shrinks by both, so the
/// eye sees `gap` between what they paint. Never below zero, so the rects
/// do not overlap.
pub(super) fn trailing_gap(width: f32, inset: f32, gap: f32) -> f32 {
    if width <= 0.0 {
        0.0
    } else if inset > 0.0 {
        (gap - inset - toolbar_end_inset()).max(0.0)
    } else {
        gap
    }
}

/// The free part of the toolbar row a [`page_header`] left this frame,
/// between the title and its own controls.
pub(super) fn toolbar_room_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::heading::toolbar_room")
}
