//! Terminal colours from the palette, so programs look native.
//!
//! The 16 ANSI colours come from the palette's roles. Entries 16 to 255
//! are xterm's cube and gray ramp.

#[cfg(test)]
mod tests;

use egui::Color32;
use toolbox_terminal::Color;

use crate::Palette;

/// xterm's six levels per channel of the 216 colour cube.
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// The ANSI colours programs print words in, as pacman and paru do: red,
/// green, yellow, blue, magenta and cyan, normal and bright.
pub const PRINTED: [u8; 12] = [1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 13, 14];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ansi([Color32; 16]);

impl Ansi {
    pub const fn new(colours: [Color32; 16]) -> Self {
        Self(colours)
    }

    /// The palette's own colours where it has them. Else black and white
    /// are mixed from the view and text colours, red, green and yellow
    /// are the status colours, blue the accent. Magenta and cyan mix the
    /// accent with red and green, as the palette has no second link
    /// colour. The bright half is lighter on a dark palette and darker on
    /// a light one.
    pub fn of(palette: &Palette, dark: bool) -> Self {
        if let Some(own) = palette.ansi {
            return own;
        }

        let base = [
            palette.view.lerp_to_gamma(palette.text, 0.25),
            palette.error,
            palette.success,
            palette.warning,
            palette.accent,
            palette.accent.lerp_to_gamma(palette.error, 0.5),
            palette.accent.lerp_to_gamma(palette.success, 0.5),
            palette.view.lerp_to_gamma(palette.text, 0.8),
        ];

        let bright = base.map(|color| {
            if dark {
                color.lerp_to_gamma(Color32::WHITE, 0.3)
            } else {
                color.lerp_to_gamma(Color32::BLACK, 0.25)
            }
        });

        let mut colours = [Color32::BLACK; 16];
        for (slot, colour) in colours.iter_mut().zip(base.into_iter().chain(bright)) {
            *slot = colour;
        }

        Self(colours)
    }

    #[must_use = "returns the changed colours, the old ones stay as they were"]
    pub fn fitted(mut self, fit: impl Fn(Color32) -> Color32) -> Self {
        for index in PRINTED {
            if let Some(slot) = self.0.get_mut(usize::from(index)) {
                *slot = fit(*slot);
            }
        }

        self
    }
}

pub fn resolve(color: Color, ansi: &Ansi, palette: &Palette) -> Color32 {
    match color {
        Color::Foreground => palette.text,
        Color::Background => palette.view,
        Color::Indexed(index) => indexed(index, ansi),
        Color::Rgb(red, green, blue) => rgb(red, green, blue),
    }
}

fn indexed(index: u8, ansi: &Ansi) -> Color32 {
    match index {
        0..=15 => ansi
            .0
            .get(usize::from(index))
            .copied()
            .unwrap_or(Color32::BLACK),
        16..=231 => {
            let cube = index - 16;
            let level = |step: u8| CUBE_LEVELS.get(usize::from(step % 6)).copied().unwrap_or(0);
            rgb(level(cube / 36), level(cube / 6), level(cube))
        }
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            rgb(gray, gray, gray)
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "terminal colours are named by the program, not the palette"
)]
const fn rgb(red: u8, green: u8, blue: u8) -> Color32 {
    Color32::from_rgb(red, green, blue)
}
