//! Every [`super::harness`] stands the skeleton's shimmer still. Its band
//! moves with the clock, so a snapshot would catch it at another place
//! in each run. Other motion stays as it is.

use egui::{Context, Id};

pub fn freeze_shimmer(ctx: &Context, frozen: bool) {
    ctx.data_mut(|data| data.insert_temp(id(), frozen));
}

pub fn shimmer_frozen(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp(id())).unwrap_or(false)
}

fn id() -> Id {
    Id::unique("toolbox_gui_kit::testing::shimmer_frozen")
}

#[cfg(test)]
mod tests;
