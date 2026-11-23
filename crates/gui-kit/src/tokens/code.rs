//! The inks of code in the guide: the colours the Console prints in,
//! fitted to the code block's fill, which is not the terminal's.

use egui::Color32;
use toolbox_terminal::Color;

use super::SECONDARY_CONTRAST;
use super::readability::{Ends, readable};
use crate::palette::Palette;
use crate::terminal_colors::{Ansi, resolve};

const GREEN: u8 = 2;
const BLUE: u8 = 4;
const MAGENTA: u8 = 5;
/// What terminals dim a prompt or a comment with.
const BRIGHT_BLACK: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeInks {
    pub plain: Color32,
    pub keyword: Color32,
    pub function: Color32,
    pub string: Color32,
    pub dim: Color32,
}

impl CodeInks {
    pub(super) fn derive(
        ansi: &Ansi,
        palette: &Palette,
        [text, field]: [Color32; 2],
        ends: Ends,
    ) -> Self {
        let ink = |index| {
            let printed = resolve(Color::Indexed(index), ansi, palette);
            readable(printed, field, SECONDARY_CONTRAST, ends)
        };

        Self {
            plain: readable(text, field, SECONDARY_CONTRAST, ends),
            keyword: ink(MAGENTA),
            function: ink(BLUE),
            string: ink(GREEN),
            dim: ink(BRIGHT_BLACK),
        }
    }
}
