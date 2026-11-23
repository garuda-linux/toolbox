//! Each row is one widget named for its item, with an id from the item's
//! key, so a row's switch keeps its animation and its focus when a search
//! drops the rows above it. The list takes one Tab stop, on a row, and Up,
//! Down, Home and End move it between the rows of the card. Tab reaches the
//! controls inside a row and the next card. Only the rows near the visible
//! part of the scroll view are laid out.

mod keys;
pub mod memory;
mod metrics;
mod row;
pub(super) mod span;

use egui::{AsIdSalt, Id, IdSalt, Rangef, Role, Ui, UiBuilder};
pub(super) use metrics::text_line_height;
pub use metrics::{row_height, text_block_height, text_block_height_with};
use span::Offsets;

use super::card;
use super::layout::SETTLED;
use super::roving::{Axes, Roving, Setup};

pub trait ListItem {
    /// Unique within the list and the same from frame to frame, such as
    /// a package or unit name.
    fn key(&self) -> &str;

    /// What a screen reader says for the row: its title, and its state
    /// where the row's controls do not say it.
    fn name(&self) -> &str;
}

/// What a list shows, row by row. A slice is one, and a page with many
/// rows builds only those asked for, so a frame costs the rows on screen.
pub trait Rows {
    type Item<'a>: ListItem
    where
        Self: 'a;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, index: usize) -> Option<Self::Item<'_>>;
}

impl<T: ListItem + ?Sized> ListItem for &T {
    fn key(&self) -> &str {
        (**self).key()
    }

    fn name(&self) -> &str {
        (**self).name()
    }
}

impl<T: ListItem> Rows for [T] {
    type Item<'a>
        = &'a T
    where
        T: 'a;

    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn get(&self, index: usize) -> Option<&T> {
        <[T]>::get(self, index)
    }
}

impl<T: ListItem, const N: usize> Rows for [T; N] {
    type Item<'a>
        = &'a T
    where
        T: 'a;

    fn len(&self) -> usize {
        N
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }
}

impl<T: ListItem> Rows for Vec<T> {
    type Item<'a>
        = &'a T
    where
        T: 'a;

    fn len(&self) -> usize {
        Self::len(self)
    }

    fn get(&self, index: usize) -> Option<&T> {
        <[T]>::get(self, index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RowHeights {
    Fixed(f32),
    /// Each row as tall as its content, remembered from the frame it was
    /// last laid out. `estimate` stands in until then.
    Measured {
        estimate: f32,
    },
}

#[derive(Clone, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "a row keeps each key and state as a flag of its own"
)]
pub struct RowState {
    pub index: usize,
    /// The row's id, which its [`ActionRow`](super::ActionRow) and
    /// [`split_row_in`](super::split_row_in) keep their sizes under.
    pub row: Id,
    pub response: egui::Response,
    /// Space was pressed on the focused row, or its empty part was
    /// clicked: its switch flips.
    pub toggle: bool,
    /// The Menu key or Shift+F10 was pressed on the focused row.
    pub menu: bool,
    /// The row is the list's selected one, drawn with the selection fill.
    pub selected: bool,
    /// The arrow keys moved the Tab stop onto this row in this frame.
    pub keyed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct RowList<'a> {
    salt: IdSalt,
    name: &'a str,
    heights: RowHeights,
    reveal: Option<&'a str>,
    selected: Option<&'a str>,
}

impl<'a> RowList<'a> {
    /// `salt` tells it apart from the other lists of the page.
    #[must_use = "the list does nothing until shown"]
    pub fn new(salt: impl AsIdSalt, name: &'a str, heights: RowHeights) -> Self {
        Self {
            salt: IdSalt::new(salt),
            name,
            heights,
            reveal: None,
            selected: None,
        }
    }

    /// Marks the row of `key` as selected, the one a details pane shows.
    #[must_use = "the list does nothing until shown"]
    pub const fn selected(mut self, key: Option<&'a str>) -> Self {
        self.selected = key;
        self
    }

    /// Focuses the row of `key` and scrolls it into view, such as an app
    /// picked in the palette. Pass it for one frame only.
    #[must_use = "the list does nothing until shown"]
    pub const fn reveal(mut self, key: Option<&'a str>) -> Self {
        self.reveal = key;
        self
    }

    /// Returns whether the row [`Self::reveal`] asked for holds the focus
    /// since the last frame. Until it does, ask again: a dialog that just
    /// closed keeps the focus from the page for one more frame.
    pub fn show<'rows, R>(
        self,
        ui: &mut Ui,
        items: &'rows R,
        mut body: impl FnMut(&mut Ui, R::Item<'rows>, &RowState),
    ) -> bool
    where
        R: Rows + ?Sized,
    {
        let list = ui.make_persistent_id(self.salt);
        memory::sweep(ui, list, items);
        let builder = UiBuilder::new()
            .accessibility_role(Role::List)
            .accessibility_label(self.name);

        card(ui, |ui| {
            ui.scope_builder(builder, |ui| {
                let (roving, holds) = stop_for_frame(ui, list, items, self.reveal);
                let keys = keys::take(ui, roving.held());
                let stop = roving.stop();
                let selected = self.selected;
                let offsets = self.offsets(ui, list, items);
                let fixed = match self.heights {
                    RowHeights::Fixed(height) => Some(height),
                    RowHeights::Measured { estimate: _ } => None,
                };

                let start = ui.cursor().top();
                let clip = ui.clip_rect();
                let visible = Rangef::new(clip.top() - start, clip.bottom() - start);
                let mut at = 0.0;
                for index in offsets.rows_to_draw(visible, Some(stop)) {
                    let Some(item) = items.get(index) else {
                        continue;
                    };

                    let id = row_id(list, item.key());
                    let top = offsets.top(index);
                    if top > at {
                        ui.add_space(top - at);
                        at = top;
                    }

                    let asked = roving.held() == Some(index);
                    let place = row::Place {
                        list,
                        id,
                        index,
                        fixed,
                        focusable: roving.is_stop(index),
                        toggle: asked && keys.toggle,
                        menu: asked && keys.menu,
                        selected: selected == Some(item.key()),
                        keyed: roving.moved() && index == stop,
                    };

                    let (height, response) = row::show(ui, place, item, &mut body);
                    roving.apply(ui, index, id, &response);
                    check_place(ui, list, (&offsets, index), height);
                    at += height;
                }

                let total = offsets.total();
                if total > at {
                    ui.add_space(total - at);
                }

                holds
            })
            .inner
        })
        .inner
    }

    fn offsets<R>(self, ui: &Ui, list: Id, items: &R) -> Offsets
    where
        R: Rows + ?Sized,
    {
        match self.heights {
            RowHeights::Fixed(height) => Offsets::Fixed {
                height,
                count: items.len(),
            },
            RowHeights::Measured { estimate } => {
                let measuring = Measuring {
                    count: items.len(),
                    estimate: estimate.to_bits(),
                    seen: row::heights_seen(ui, list),
                };

                let kept_id = list.with("offsets");
                let kept = ui.data(|data| data.get_temp::<(Measuring, Offsets)>(kept_id));
                if let Some((was, offsets)) = kept
                    && was == measuring
                {
                    return offsets;
                }

                let offsets = Offsets::measured((0..items.len()).map(|index| {
                    items
                        .get(index)
                        .and_then(|item| row::remembered(ui, row_id(list, item.key())))
                        .unwrap_or(estimate)
                }));

                ui.data_mut(|data| data.insert_temp(kept_id, (measuring, offsets.clone())));
                offsets
            }
        }
    }
}

/// What a list's measured offsets come from besides the rows' heights,
/// whose changes [`row::heights_seen`] counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Measuring {
    count: usize,
    estimate: u32,
    seen: u64,
}

/// A drawn row whose height is not its place in the kept offsets, as
/// after rows swapped places, makes them stale.
fn check_place(ui: &Ui, list: Id, (offsets, index): (&Offsets, usize), height: f32) {
    let slot = offsets.top(index.saturating_add(1)) - offsets.top(index);
    if matches!(offsets, Offsets::Measured { bottoms: _ }) && (slot - height).abs() >= SETTLED {
        row::heights_changed(ui, list);
        ui.ctx().request_repaint();
    }
}

/// The list's Tab stop for this frame. A row the page reveals takes the stop
/// and the focus until it holds them, and `holds` tells whether it does.
fn stop_for_frame<R>(ui: &Ui, list: Id, items: &R, reveal: Option<&str>) -> (Roving, bool)
where
    R: Rows + ?Sized,
{
    let mut roving = Roving::read(
        ui,
        list,
        items.len(),
        |index| id_at(list, items, index),
        Setup {
            rest: None,
            axes: Axes::Vertical,
        },
    );

    // Only for the frame a page asks to reveal a row.
    let revealed =
        reveal.and_then(|key| position(items, |item| item.key() == key).map(|at| (key, at)));

    let holds =
        revealed.is_some_and(|(key, _)| ui.memory(|memory| memory.has_focus(row_id(list, key))));

    if let Some((_, index)) = revealed.filter(|_| !holds) {
        roving.focus(ui, index);
    }

    (roving, holds)
}

/// The row's id for the Tab stop, the one the stop's index names.
fn id_at<R>(list: Id, items: &R, index: usize) -> Id
where
    R: Rows + ?Sized,
{
    items
        .get(index)
        .map_or(list, |item| row_id(list, item.key()))
}

fn position<R>(items: &R, mut found: impl FnMut(&R::Item<'_>) -> bool) -> Option<usize>
where
    R: Rows + ?Sized,
{
    (0..items.len()).find(|&index| items.get(index).is_some_and(|item| found(&item)))
}

fn row_id(list: Id, key: &str) -> Id {
    list.with(key)
}
