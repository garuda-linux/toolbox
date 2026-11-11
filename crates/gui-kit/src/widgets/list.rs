//! Each row is one focusable widget named for its item, with an id from
//! the item's key, so a row's switch keeps its animation and its focus
//! when a search drops the rows above it. Up and Down move between the
//! rows of one card, Tab between the controls and cards. Only the rows
//! near the visible part of the scroll view are laid out.

mod keys;
mod metrics;
mod row;
pub(super) mod span;

use egui::{AsIdSalt, Id, IdSalt, Rangef, Role, Ui, UiBuilder};
pub(super) use metrics::text_line_height;
pub use metrics::{row_height, text_block_height};
use span::Offsets;

use super::card;

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
pub struct RowState {
    pub index: usize,
    pub response: egui::Response,
    /// Space was pressed on the focused row, or its empty part was
    /// clicked: its switch flips.
    pub toggle: bool,
    /// The Menu key or Shift+F10 was pressed on the focused row.
    pub menu: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct RowList<'a> {
    salt: IdSalt,
    name: &'a str,
    heights: RowHeights,
    reveal: Option<&'a str>,
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
        }
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
        let builder = UiBuilder::new()
            .accessibility_role(Role::List)
            .accessibility_label(self.name);

        card(ui, |ui| {
            ui.scope_builder(builder, |ui| {
                // Only for the frame a page asks to reveal a row.
                let revealed = self
                    .reveal
                    .filter(|key| position(items, |item| item.key() == *key).is_some());

                let holds = revealed
                    .is_some_and(|key| ui.memory(|memory| memory.has_focus(row_id(list, key))));

                if let Some(key) = revealed.filter(|_| !holds) {
                    ui.memory_mut(|memory| memory.request_focus(row_id(list, key)));
                }

                let (current, keys) = navigate(ui, list, items);
                let offsets = self.offsets(ui, list, items);
                let fixed = match self.heights {
                    RowHeights::Fixed(height) => Some(height),
                    RowHeights::Measured { estimate: _ } => None,
                };

                let start = ui.cursor().top();
                let clip = ui.clip_rect();
                let visible = Rangef::new(clip.top() - start, clip.bottom() - start);
                let mut at = 0.0;
                for index in offsets.rows_to_draw(visible, current) {
                    let Some(item) = items.get(index) else {
                        continue;
                    };

                    let id = row_id(list, item.key());
                    if ui.memory(|memory| memory.has_focus(id)) {
                        remember_focused(ui, list, index);
                    }

                    let top = offsets.top(index);
                    if top > at {
                        ui.add_space(top - at);
                        at = top;
                    }

                    let asked = keys.moved_to.is_none() && current == Some(index);
                    let place = row::Place {
                        id,
                        index,
                        fixed,
                        toggle: asked && keys.toggle,
                        menu: asked && keys.menu,
                    };

                    at += row::show(ui, place, item, &mut body);
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
            RowHeights::Measured { estimate } => Offsets::measured((0..items.len()).map(|index| {
                items
                    .get(index)
                    .and_then(|item| row::remembered(ui, row_id(list, item.key())))
                    .unwrap_or(estimate)
            })),
        }
    }
}

fn navigate<R>(ui: &Ui, list: Id, items: &R) -> (Option<usize>, keys::RowKeys)
where
    R: Rows + ?Sized,
{
    let focused = ui.memory(egui::Memory::focused);
    let current = focused.and_then(|focused| focused_index(ui, list, items, focused));

    let Some(current) = current else {
        return (None, keys::RowKeys::default());
    };

    let keys = keys::take(ui, current, items.len());
    let Some(target) = keys.moved_to else {
        return (Some(current), keys);
    };

    if let Some(item) = items.get(target) {
        let id = row_id(list, item.key());
        ui.memory_mut(|memory| memory.request_focus(id));
        remember_focused(ui, list, target);
    }

    (Some(target), keys)
}

/// The index of the focused row. The one remembered when it took the
/// focus is checked first, so a frame reads one row, not all of them.
fn focused_index<R>(ui: &Ui, list: Id, items: &R, focused: Id) -> Option<usize>
where
    R: Rows + ?Sized,
{
    let remembered = ui.data(|data| data.get_temp::<usize>(focus_memory(list)));
    let still = remembered.filter(|&index| {
        items
            .get(index)
            .is_some_and(|item| row_id(list, item.key()) == focused)
    });

    if still.is_some() {
        return still;
    }

    // The focus is in another widget, or the rows moved under it. One
    // search per focus change and per row count: a row that is focused
    // later is drawn and remembered first.
    let searched = list.with("focus-searched");
    let asked = (focused, items.len());
    if ui.data(|data| data.get_temp::<(Id, usize)>(searched)) == Some(asked) {
        return None;
    }

    ui.data_mut(|data| data.insert_temp(searched, asked));
    position(items, |item| row_id(list, item.key()) == focused)
}

fn remember_focused(ui: &Ui, list: Id, index: usize) {
    ui.data_mut(|data| data.insert_temp(focus_memory(list), index));
}

fn focus_memory(list: Id) -> Id {
    list.with("focused-row")
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
