//! The zoom of the whole UI, by Ctrl and plus, minus or 0 as in a browser.
//! egui's own zoom keys stay off: they would take the keys from a focused
//! terminal and zoom without bounds.

#[cfg(test)]
mod tests;

use egui::{Context, Id, Vec2, ViewportCommand, vec2};
use serde::{Deserialize, Serialize};

/// In points. Every page is laid out down to it.
pub const SMALLEST_WINDOW: Vec2 = vec2(640.0, 480.0);

/// Below 80 % the caption text gets too small to read.
const FEWEST_TENTHS: i8 = -2;
/// Above 150 % the smallest window, which grows with the zoom, no longer
/// fits a 1366 × 768 screen.
const MOST_TENTHS: i8 = 5;

/// The zoom in tenths away from 100 %, so steps add up without drift.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "f32", into = "f32")]
pub struct Zoom {
    tenths: i8,
}

impl Zoom {
    #[must_use = "it only computes the next zoom"]
    pub const fn larger(self) -> Self {
        Self::clamped(self.tenths.saturating_add(1))
    }

    #[must_use = "it only computes the next zoom"]
    pub const fn smaller(self) -> Self {
        Self::clamped(self.tenths.saturating_sub(1))
    }

    #[must_use = "it only answers a question"]
    pub fn factor(self) -> f32 {
        f32::from(self.tenths).mul_add(0.1, 1.0)
    }

    const fn clamped(tenths: i8) -> Self {
        let tenths = if tenths < FEWEST_TENTHS {
            FEWEST_TENTHS
        } else if tenths > MOST_TENTHS {
            MOST_TENTHS
        } else {
            tenths
        };

        Self { tenths }
    }

    pub fn apply(self, ctx: &Context) {
        ctx.set_zoom_factor(self.factor());
    }
}

/// A stored factor from another version, or edited by hand, comes back
/// rounded to a tenth and within the bounds.
impl From<f32> for Zoom {
    fn from(factor: f32) -> Self {
        let tenths = ((factor - 1.0) * 10.0)
            .round()
            .clamp(f32::from(FEWEST_TENTHS), f32::from(MOST_TENTHS));

        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "clamped to a few tenths above, NaN turns into 0"
        )]
        let tenths = tenths as i8;
        Self { tenths }
    }
}

impl From<Zoom> for f32 {
    fn from(zoom: Zoom) -> Self {
        zoom.factor()
    }
}

pub fn install(ctx: &Context, zoom: Zoom) {
    ctx.options_mut(|options| options.zoom_with_keyboard = false);
    zoom.apply(ctx);
}

/// The window takes the size in pixels at the zoom of the frame that sends it,
/// and a new zoom only holds from the frame after the change, so this
/// runs on the first frame that shows a new zoom.
pub fn keep_smallest_window(ctx: &Context) {
    let zoom = Zoom::from(ctx.zoom_factor());
    let id = Id::unique("toolbox_gui::zoom::smallest_window");
    let sent_for = ctx.data_mut(|data| {
        let sent_for = data.get_temp::<Zoom>(id);
        data.insert_temp(id, zoom);
        sent_for
    });

    if sent_for != Some(zoom) {
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(SMALLEST_WINDOW));
    }
}
