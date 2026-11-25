//! The shapes of a page's body: one column in a scroll area, the same
//! column filling the window down to a card's gap, a full-width list, or
//! wide panes that scroll on their own.

use egui::{Rect, ScrollArea, Ui, UiBuilder, pos2};

use super::{column, column_rect};
use crate::motion;
use crate::theme::show_scroll;
use crate::tokens::Sizes;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BodyShape {
    /// The usual page: the whole body, headings and notes included, fills
    /// the page's width less its margins.
    #[default]
    Column,
    /// A column whose last block fills the window. It ends `card_gap`
    /// above the bottom, the gap between its cards, not the page padding
    /// a column that scrolls keeps under its end.
    Filled,
    /// A list page (packages, services, the App Catalog, config files)
    /// takes the window's full width, as Kirigami's list pages do.
    Wide,
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
    /// The widest the body's column gets, the window's full width for a list.
    pub widest: f32,
}

impl BodyShape {
    /// The room under the body's end.
    pub(super) fn bottom(self, sizes: &Sizes) -> f32 {
        match self {
            Self::Filled => sizes.card_gap,
            Self::Column | Self::Wide | Self::Panes { width: _ } => {
                f32::from(sizes.page_padding.bottom)
            }
        }
    }

    /// The widest the body's column gets. A list has no limit beyond the
    /// window, so its column fills the page's padded width.
    pub(super) const fn width(self) -> f32 {
        match self {
            Self::Column | Self::Filled | Self::Wide => f32::INFINITY,
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
) -> f32 {
    let mut body_ui = ui.new_child(UiBuilder::new().max_rect(room.view));
    let scroll = show_scroll(&mut body_ui, area, |scrolling| {
        // The page runs on under the header. Only the painting reaches up
        // there: the area itself starts below the band, so a control
        // scrolled into sight lands fully below it.
        let mut under = scrolling.clip_rect();
        under.min.y = room.page_top;
        under.max.y = room.floor;
        scrolling.set_clip_rect(under);

        let rise = (1.0 - room.entered) * motion::PAGE_RISE;
        column(
            scrolling,
            room.top + rise,
            room.bottom,
            room.widest,
            |inner| {
                inner.multiply_opacity(room.entered);
                body(inner);
            },
        );
    });

    remember_overflow(ui.ctx(), scroll.content_size.y > scroll.inner_rect.height());
    let offset = scroll.state.clamped_offset().y;
    scroll.content_size.y - offset - scroll.inner_rect.height()
}

/// Whether the body's content outgrew its view in the frame it drew. The
/// header is drawn before the body, so it reads the last frame's answer.
fn remember_overflow(ctx: &egui::Context, overflowed: bool) {
    ctx.data_mut(|data| data.insert_temp(overflow_id(), overflowed));
}

/// Whether the body overflowed last frame, which sets whether a scroll strip
/// takes room at the right.
pub(super) fn overflowed(ctx: &egui::Context) -> bool {
    ctx.data(|data| data.get_temp(overflow_id()))
        .unwrap_or(false)
}

fn overflow_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::page::overflow")
}
