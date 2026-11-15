//! The value a [`question`](super::question) asks about, held in the frame's
//! temp data so the page keeps no dialog state of its own.

use std::marker::PhantomData;

use egui::{Context, Id};

/// A value held under `id` from the moment a question opens until the
/// question is over. Its dialog is `id` with "dialog" added.
#[derive(Clone, Copy, Debug)]
pub struct Pending<T> {
    id: Id,
    held: PhantomData<fn() -> T>,
}

impl<T: Clone + Send + Sync + 'static> Pending<T> {
    #[must_use = "a pending value does nothing until it is held"]
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            held: PhantomData,
        }
    }

    /// The id a [`question`](super::question) of this value takes.
    #[must_use = "the id does nothing until a question takes it"]
    pub fn dialog(&self) -> Id {
        self.id.with("dialog")
    }

    pub fn hold(&self, ctx: &Context, value: T) {
        ctx.data_mut(|data| data.insert_temp(self.id, value));
    }

    #[must_use = "the held value is only read here"]
    pub fn get(&self, ctx: &Context) -> Option<T> {
        ctx.data(|data| data.get_temp::<T>(self.id))
    }

    /// Drops the value, as when the question is answered or the page is left.
    pub fn release(&self, ctx: &Context) {
        ctx.data_mut(|data| data.remove::<T>(self.id));
    }
}
