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

impl<T: ListItem + ?Sized> ListItem for &T {
    fn key(&self) -> &str {
        (**self).key()
    }

    fn name(&self) -> &str {
        (**self).name()
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
    pub fn show<T>(
        self,
        ui: &mut Ui,
        items: &[T],
        mut body: impl FnMut(&mut Ui, &T, &RowState),
    ) -> bool
    where
        T: ListItem,
    {
        let list = ui.make_persistent_id(self.salt);
        let builder = UiBuilder::new()
            .accessibility_role(Role::List)
            .accessibility_label(self.name);

        card(ui, |ui| {
            ui.scope_builder(builder, |ui| {
                let revealed = self
                    .reveal
                    .filter(|key| items.iter().any(|item| item.key() == *key));

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

                    let top = offsets.top(index);
                    if top > at {
                        ui.add_space(top - at);
                        at = top;
                    }

                    let asked = keys.moved_to.is_none() && current == Some(index);
                    let place = row::Place {
                        id: row_id(list, item.key()),
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

    fn offsets<T>(self, ui: &Ui, list: Id, items: &[T]) -> Offsets
    where
        T: ListItem,
    {
        match self.heights {
            RowHeights::Fixed(height) => Offsets::Fixed {
                height,
                count: items.len(),
            },
            RowHeights::Measured { estimate } => Offsets::measured(
                items
                    .iter()
                    .map(|item| row::remembered(ui, row_id(list, item.key())).unwrap_or(estimate)),
            ),
        }
    }
}

fn navigate<T>(ui: &Ui, list: Id, items: &[T]) -> (Option<usize>, keys::RowKeys)
where
    T: ListItem,
{
    let focused = ui.memory(egui::Memory::focused);
    let current = focused.and_then(|focused| {
        items
            .iter()
            .position(|item| row_id(list, item.key()) == focused)
    });

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
    }

    (Some(target), keys)
}

fn row_id(list: Id, key: &str) -> Id {
    list.with(key)
}
