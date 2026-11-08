//! The sidebar folds by itself in a narrow window, so the page keeps room
//! to read, and unfolds when the window is wide again, but only when it
//! folded itself. What the user chose with Ctrl+B stays.

#[cfg(test)]
mod tests;

/// Below this window width, in points, the sidebar folds.
pub const NARROW: f32 = 720.0;

#[derive(Debug, Default)]
pub struct Fold {
    narrow: bool,
    folded: bool,
}

impl Fold {
    /// Whether the sidebar is expanded in a window `width` wide, from
    /// whether it is now.
    pub fn fit(&mut self, width: f32, expanded: bool) -> bool {
        let narrow = width < NARROW;
        let fitted = match (self.narrow, narrow) {
            (false, true) if expanded => {
                self.folded = true;
                false
            }
            (true, false) if self.folded => {
                self.folded = false;
                true
            }
            (true, true) if expanded => {
                // The user unfolded it, which this does not undo.
                self.folded = false;
                true
            }
            _ => expanded,
        };

        self.narrow = narrow;
        fitted
    }
}
