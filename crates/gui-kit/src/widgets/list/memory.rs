//! What a list keeps of its rows between frames: one struct per row id,
//! dropped once the row left the list's items, with the marks the row's
//! ui left under its id.

use std::hash::{DefaultHasher, Hash as _, Hasher as _};

use egui::{Id, IdSet, Ui};

use super::super::card::forget_raised;
use super::{ListItem as _, Rows, row_id};
use crate::motion;

/// What one row keeps between frames.
#[derive(Clone, Copy, Debug, Default)]
pub struct RowMemory {
    /// Its height when last laid out, the divider above included.
    pub height: Option<f32>,
    /// How tall the content of a fixed height row was, to centre it.
    pub content: Option<f32>,
}

/// The rows of one list that keep memory, and the item set they were
/// kept for.
#[derive(Clone, Debug, Default)]
struct Kept {
    items: u64,
    rows: IdSet,
}

pub fn get(ui: &Ui, row: Id) -> RowMemory {
    ui.data(|data| data.get_temp::<RowMemory>(memory_id(row)))
        .unwrap_or_default()
}

pub fn update(ui: &Ui, (list, row): (Id, Id), change: impl FnOnce(&mut RowMemory)) {
    ui.data_mut(|data| {
        change(data.get_temp_mut_or_default::<RowMemory>(memory_id(row)));
        data.get_temp_mut_or_default::<Kept>(kept_id(list))
            .rows
            .insert(row);
    });
}

/// Drops what the rows that left `items` kept, once the item set
/// changed. The set is told by its length and its first and last keys,
/// so a frame reads two rows, not all of them.
pub fn sweep<R>(ui: &Ui, list: Id, items: &R)
where
    R: Rows + ?Sized,
{
    let now = fingerprint(items);
    let kept = ui.data_mut(|data| {
        let kept = data.get_temp_mut_or_default::<Kept>(kept_id(list));
        if kept.items == now {
            return None;
        }

        kept.items = now;
        Some(std::mem::take(&mut kept.rows))
    });

    let Some(rows) = kept.filter(|rows| !rows.is_empty()) else {
        return;
    };

    let live: IdSet = (0..items.len())
        .filter_map(|index| items.get(index).map(|item| row_id(list, item.key())))
        .collect();

    let (stay, gone): (IdSet, IdSet) = rows.into_iter().partition(|row| live.contains(row));
    ui.data_mut(|data| {
        for row in &gone {
            data.remove::<RowMemory>(memory_id(*row));
        }

        data.get_temp_mut_or_default::<Kept>(kept_id(list))
            .rows
            .extend(stay);
    });

    for row in gone {
        forget_raised(ui, row);
        motion::forget_keyboard(ui.ctx(), row);
    }
}

fn fingerprint<R>(items: &R) -> u64
where
    R: Rows + ?Sized,
{
    let len = items.len();
    let key = |index: usize| items.get(index).map(|item| item.key().to_owned());
    let last = len.checked_sub(1).and_then(key);
    let mut hasher = DefaultHasher::new();
    (len, key(0), last).hash(&mut hasher);
    hasher.finish()
}

fn memory_id(row: Id) -> Id {
    row.with("toolbox_gui_kit::row_memory")
}

fn kept_id(list: Id) -> Id {
    list.with("toolbox_gui_kit::rows_kept")
}
