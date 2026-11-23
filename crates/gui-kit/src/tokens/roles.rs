//! The desktop roles that only some surfaces take: Kirigami's active text,
//! which information banners take, and the Complementary set, which the
//! passive notes take.

use egui::Color32;

use super::readability::Ends;
use super::{SECONDARY_CONTRAST, Surfaces, TEXT_CONTRAST, fitted, readable};
use crate::palette::Palette;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Roles {
    /// The view's active text colour. The accent where the desktop names none.
    pub active: Color32,
    /// The Complementary set's background, the window where the desktop names none.
    pub complementary: Color32,
    /// The Complementary set's text, the text where the desktop names none.
    pub complementary_text: Color32,
}

pub(super) fn derive(
    palette: &Palette,
    surfaces: &Surfaces,
    accent: Color32,
    ink: Color32,
    ends: Ends,
) -> Roles {
    let note = palette.complementary.unwrap_or(surfaces.window);
    Roles {
        active: fitted(
            palette.active.unwrap_or(accent),
            &surfaces.under_text(),
            SECONDARY_CONTRAST,
            ends,
        ),
        complementary: note,
        complementary_text: readable(
            palette.complementary_text.unwrap_or(ink),
            note,
            TEXT_CONTRAST,
            ends,
        ),
    }
}
