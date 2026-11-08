use egui::Context;
pub use toolbox_services::waking::{Wake, Waking, channel};

/// The window's wake: a repaint, so the frame takes the new values.
#[must_use = "the wake is given to a service"]
pub fn repaint(ctx: Context) -> Wake {
    Wake::new(move || ctx.request_repaint())
}
