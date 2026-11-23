//! What the desktop says about its look, parsed from text the app reads:
//! `kdeglobals` on KDE and LXQt, `gtk.css` on GTK desktops. Parsing only,
//! the app does the reading and watching.

pub mod family;
pub mod gtk;
mod ini;
pub mod kde;
pub mod look;
#[cfg(test)]
mod tests;

use egui::Color32;

use crate::Palette;

/// The colours one desktop file sets. A missing role keeps the colour of
/// the palette it is laid over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub window: Option<Color32>,
    pub view: Option<Color32>,
    pub button: Option<Color32>,
    pub text: Option<Color32>,
    pub accent: Option<Color32>,
    pub on_accent: Option<Color32>,
    pub error: Option<Color32>,
    pub warning: Option<Color32>,
    pub success: Option<Color32>,
    pub link: Option<Color32>,
    pub visited: Option<Color32>,
    pub active: Option<Color32>,
    pub complementary: Option<Color32>,
    pub complementary_text: Option<Color32>,
    pub decoration_hover: Option<Color32>,
    pub decoration_focus: Option<Color32>,
    pub tooltip: Option<Color32>,
    pub tooltip_text: Option<Color32>,
    pub contrast: Option<i8>,
}

/// A font as the desktop names it. The app finds the file.
#[derive(Clone, Debug, PartialEq)]
pub struct FontSpec {
    pub family: String,
    pub points: f32,
    /// CSS weight, 100 to 900.
    pub weight: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopTheme {
    pub colors: Colors,
    pub font: Option<FontSpec>,
    pub fixed_font: Option<FontSpec>,
    pub icon_theme: Option<String>,
    /// `Some(false)` when the desktop turned its animations off.
    pub animations: Option<bool>,
}

impl Colors {
    /// `base` with every role this file sets replaced.
    pub fn over(&self, base: Palette) -> Palette {
        Palette {
            window: self.window.unwrap_or(base.window),
            view: self.view.unwrap_or(base.view),
            button: self.button.or(base.button),
            text: self.text.unwrap_or(base.text),
            accent: self.accent.unwrap_or(base.accent),
            on_accent: self.on_accent.unwrap_or(base.on_accent),
            error: self.error.unwrap_or(base.error),
            warning: self.warning.unwrap_or(base.warning),
            success: self.success.unwrap_or(base.success),
            link: self.link.or(base.link),
            visited: self.visited.or(base.visited),
            active: self.active.or(base.active),
            complementary: self.complementary.or(base.complementary),
            complementary_text: self.complementary_text.or(base.complementary_text),
            decoration_hover: self.decoration_hover.or(base.decoration_hover),
            decoration_focus: self.decoration_focus.or(base.decoration_focus),
            tooltip: self.tooltip.or(base.tooltip),
            tooltip_text: self.tooltip_text.or(base.tooltip_text),
            contrast: self.contrast.or(base.contrast),
            tiles: base.tiles,
            ansi: base.ansi,
        }
    }
}
