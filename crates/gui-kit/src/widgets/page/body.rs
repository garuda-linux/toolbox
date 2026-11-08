//! The two shapes of a page's body: one column in a scroll area, or wide
//! panes that scroll on their own.

use egui::{Rect, ScrollArea, Ui, UiBuilder, pos2};

use super::{column, column_rect};
use crate::motion;
use crate::theme::CONTENT_WIDTH;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BodyShape {
    #[default]
    Column,
    /// Up to `width` wide and as tall as the page, without a scroll area,
    /// for a page whose panes scroll on their own, such as a reader with
    /// its contents beside the text.
    Panes { width: f32 },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Room {
    pub view: Rect,
    pub top: f32,
    pub bottom: f32,
    /// The top of the whole page, which the column paints up to.
    pub page_top: f32,
    pub floor: f32,
    /// How far the body has come in, 0 to 1.
    pub entered: f32,
}

impl BodyShape {
    pub(super) const fn width(self) -> f32 {
        match self {
            Self::Column => CONTENT_WIDTH,
            Self::Panes { width } => width,
        }
    }
}

pub(super) fn panes(ui: &mut Ui, room: Room, widest: f32, body: impl FnOnce(&mut Ui)) {
    let pane = Rect::from_min_max(
        pos2(room.view.left(), room.view.top() + room.top),
        pos2(room.view.right(), room.view.bottom() - room.bottom),
    );

    let rect = column_rect(ui, pane, 0.0, widest);
    let full = Rect::from_min_max(rect.min, pos2(rect.max.x, pane.bottom()));

    let mut inner = ui.new_child(UiBuilder::new().max_rect(full));
    inner.multiply_opacity(room.entered);
    body(&mut inner);
}

pub(super) fn scrolled(
    ui: &mut Ui,
    area: ScrollArea,
    room: Room,
    body: impl FnOnce(&mut Ui),
) -> (f32, f32) {
    let mut body_ui = ui.new_child(UiBuilder::new().max_rect(room.view));
    let scroll = area.show(&mut body_ui, |scrolling| {
        // The page runs on under the header. Only the painting reaches up
        // there: the area itself starts below the band, so a control
        // scrolled into sight lands fully below it.
        let mut under = scrolling.clip_rect();
        under.min.y = room.page_top;
        under.max.y = room.floor;
        scrolling.set_clip_rect(under);

        let rise = (1.0 - room.entered) * motion::PAGE_RISE;
        column(scrolling, room.top + rise, room.bottom, |inner| {
            inner.multiply_opacity(room.entered);
            body(inner);
        });
    });

    let offset = scroll.state.clamped_offset().y;
    let hidden = scroll.content_size.y - offset - scroll.inner_rect.height();
    (offset, hidden)
}
