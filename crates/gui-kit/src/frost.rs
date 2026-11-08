//! Frosted glass: a surface that blurs what lies under it, live, under the
//! tint that keeps its text readable.
//!
//! Menus, notes, dialogs and the Console use it. The page header blurs
//! live too, under a neutral scrim instead.
//!
//! The blur needs eframe's wgpu renderer ([`install`]). Without it a
//! surface keeps its tint alone, thick enough that sharp content under it
//! stays a faint ghost.

mod live;
mod surface;
#[cfg(test)]
mod tests;

use egui::{Color32, Context};
pub(crate) use live::reserve;
pub use live::{backdrop, backdrop_within, install, live};
pub use surface::Glass;

use crate::tokens;

/// The tint of menus, lists, notes and popovers.
pub fn raised(ctx: &Context) -> Color32 {
    let tokens = tokens::current(ctx);
    if live(ctx) {
        tokens.frost_raised_blurred
    } else {
        tokens.frost_raised
    }
}
