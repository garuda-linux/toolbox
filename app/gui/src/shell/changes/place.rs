//! Which side of its pill the popover opens on, so it never runs into
//! the window's edge.

use egui::{Align2, Pos2, Rect};

/// The popover's height in parts, as the last frame measured it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fit {
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

impl Place {
    /// The popover's corner at `pill` and its pivot. `rise` moves it
    /// toward the pill while it fades in.
    pub fn corner(self, pill: Rect, gap: f32, rise: f32) -> (Pos2, Align2) {
        if self.above {
            (
                Pos2::new(pill.right(), pill.top() - gap + rise),
                Align2::RIGHT_BOTTOM,
            )
        } else {
            (
                Pos2::new(pill.right(), pill.bottom() + gap - rise),
                Align2::RIGHT_TOP,
            )
        }
    }
}

/// Above the pill when the popover fits there, else below when it fits
/// there, else on the roomier side with a shorter list. `gap` is kept to
/// the pill and to the window's edge.
pub fn place(pill: Rect, window: Rect, fit: Fit, gap: f32) -> Place {
    let height = fit.chrome + fit.list.min(fit.most);
    let edges = gap + gap;
    let above = pill.top() - window.top() - edges;
    let below = window.bottom() - pill.bottom() - edges;
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
