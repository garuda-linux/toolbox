//! Which side of Review the popover opens on, so it never runs into the
//! window's edge.

use egui::Rect;

/// The popover's height in parts, as the last frame measured it.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Fit {
    /// All but the list.
    pub chrome: f32,
    /// The list's whole height, scrolled or not.
    pub list: f32,
    /// The list's height before it scrolls.
    pub most: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub above: bool,
    /// The list's height before it scrolls here.
    pub list: f32,
}

/// Above Review when the popover fits there, else below when it fits
/// there, else on the roomier side with a shorter list. `gap` is kept to
/// Review and to the window's edge.
pub fn place(review: Rect, window: Rect, fit: Fit, gap: f32) -> Place {
    let height = fit.chrome + fit.list.min(fit.most);
    let edges = gap + gap;
    let above = review.top() - window.top() - edges;
    let below = window.bottom() - review.bottom() - edges;
    if height <= above || (height > below && above >= below) {
        return Place {
            above: true,
            list: shortened(fit, above),
        };
    }

    Place {
        above: false,
        list: shortened(fit, below),
    }
}

fn shortened(fit: Fit, room: f32) -> f32 {
    fit.most.min(room - fit.chrome).max(0.0)
}

#[cfg(test)]
mod tests;
