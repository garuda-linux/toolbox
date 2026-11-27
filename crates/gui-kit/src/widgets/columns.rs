//! Two columns side by side, as Kirigami's `ColumnView` lays them out. Each
//! column scrolls on its own. Below the width that fits two columns, an open
//! row shows only its details.

use egui::{Id, Rect, ScrollArea, Stroke, Ui, UiBuilder, pos2, vec2};

use crate::theme::{self, LARGE_SPACING, show_scroll};
use crate::tokens;

/// Kirigami's `PageRow.defaultColumnWidth`, which is 20 grid units.
const DEFAULT_GRID_UNITS: f32 = 20.0;

/// The narrowest a column gets, in points.
pub const MIN_COLUMN: f32 = 300.0;

/// The line between the two columns.
const LINE: f32 = 1.0;

/// The space between each column and the line.
const GAP: f32 = LARGE_SPACING;

/// The widths of the two columns for one content width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Split {
    pub list: f32,
    pub details: f32,
}

/// The list and its details in one place of a page.
#[derive(Clone, Copy, Debug)]
pub struct Columns<'a> {
    salt: &'a str,
    open: bool,
}

impl<'a> Columns<'a> {
    /// `salt` keeps the scroll positions of the two columns apart from the
    /// other places of the page.
    #[must_use = "the columns do nothing until shown"]
    pub const fn new(salt: &'a str) -> Self {
        Self { salt, open: false }
    }

    /// Whether a row is open, so the details have something to show.
    #[must_use = "the columns do nothing until shown"]
    pub const fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Draws the list and the details. `state` goes to whichever column
    /// draws, one after the other, so both can change the same page.
    pub fn show<S>(
        self,
        ui: &mut Ui,
        state: &mut S,
        list: impl FnOnce(&mut Ui, &mut S),
        details: impl FnOnce(&mut Ui, &mut S),
    ) {
        let area = ui.available_rect_before_wrap();
        let base = ui.scope_id().with(self.salt);
        let widths = if self.open {
            split(area.width(), theme::GRID_UNIT)
        } else {
            None
        };

        match widths {
            Some(split) => {
                let left = Rect::from_min_size(area.min, vec2(split.list, area.height()));
                let right = Rect::from_min_size(
                    pos2(2.0_f32.mul_add(GAP, left.right() + LINE), area.top()),
                    vec2(split.details, area.height()),
                );

                column(ui, base.with("list"), left, |ui| list(ui, state));
                let line = tokens::current(ui.ctx()).separator;
                let x = left.right() + GAP + LINE / 2.0;
                ui.painter()
                    .vline(x, area.y_range(), Stroke::new(LINE, line));
                column(ui, base.with("details"), right, |ui| details(ui, state));
            }
            None if self.open => {
                column(ui, base.with("details"), area, |ui| details(ui, state));
            }
            None => column(ui, base.with("list"), area, |ui| list(ui, state)),
        }

        ui.advance_cursor_after_rect(area);
    }
}

/// One column in its own scroll area. The child's clip is its own rectangle,
/// as the parent's clip spans both columns.
fn column(ui: &mut Ui, salt: Id, rect: Rect, body: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
    child.set_clip_rect(rect.intersect(child.clip_rect()));
    show_scroll(
        &mut child,
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_salt(salt),
        body,
    );
}

/// Whether `width` points of content leave no room for two columns.
pub fn is_narrow(width: f32) -> bool {
    split(width, theme::GRID_UNIT).is_none()
}

/// The widths for `width` points of content.
///
/// The list takes a third of the content, at least `MIN_COLUMN` and at most
/// Kirigami's default column width. The details take the rest after the gap
/// on each side of the line. `None` when the content cannot hold two minimum
/// columns, the gaps and the line.
pub fn split(width: f32, grid_unit: f32) -> Option<Split> {
    if width < 2.0_f32.mul_add(MIN_COLUMN, 2.0_f32.mul_add(GAP, LINE)) {
        return None;
    }

    let widest = (DEFAULT_GRID_UNITS * grid_unit).max(MIN_COLUMN);
    let list = (width / 3.0).clamp(MIN_COLUMN, widest);
    Some(Split {
        list,
        details: 2.0_f32.mul_add(-GAP, width - list - LINE),
    })
}
