#[cfg(test)]
mod tests;

use egui::{Color32, Theme};

use super::{DesktopTheme, FontSpec};
use crate::Palette;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Portal {
    /// `None` for "no preference" or no portal.
    pub dark: Option<bool>,
    pub accent: Option<Color32>,
    /// GNOME's `enable-animations`, `None` where the portal lacks it.
    pub animations: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sources {
    pub portal: Portal,
    /// `kdeglobals`, or GTK's `gtk.css` with `settings.ini`.
    pub desktop: Option<DesktopTheme>,
    /// GTK's `gtk-application-prefer-dark-theme`.
    pub prefer_dark: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub theme: Theme,
    pub dark: Palette,
    pub light: Palette,
    pub font: Option<FontSpec>,
    pub fixed_font: Option<FontSpec>,
    pub icon_theme: Option<String>,
    pub reduced_motion: bool,
}

impl Portal {
    /// The portal's `accent-color`: red, green and blue from 0 to 1. A
    /// value outside that range means the desktop sets no accent.
    #[expect(
        clippy::disallowed_methods,
        reason = "the palette and the desktop parsers are where colours are made"
    )]
    pub fn accent_from_unit_rgb(red: f64, green: f64, blue: f64) -> Option<Color32> {
        Some(Color32::from_rgb(
            unit_to_byte(red)?,
            unit_to_byte(green)?,
            unit_to_byte(blue)?,
        ))
    }
}

impl Look {
    /// The theme follows the desktop's own window colour first, as that is
    /// what its other apps show. The portal's accent fills in only where
    /// the desktop's files name none. The theme not in use keeps the
    /// fallback palette.
    pub fn resolve(sources: &Sources) -> Self {
        let desktop = sources.desktop.clone().unwrap_or_default();

        // A user who turned animations off in any of the places the
        // desktop keeps them wants less motion, whatever the others say.
        let reduced_motion = [desktop.animations, sources.portal.animations].contains(&Some(false));
        let theme = desktop
            .colors
            .window
            .map(|window| theme_of(window.intensity() < 0.5))
            .or_else(|| sources.portal.dark.map(theme_of))
            .or_else(|| sources.prefer_dark.map(theme_of))
            .unwrap_or(Theme::Dark);

        let mut colors = desktop.colors;
        if let (None, Some(accent)) = (colors.accent, sources.portal.accent) {
            colors.accent = Some(accent);
            colors.on_accent = Some(readable_on(accent));
        }

        let active = colors.over(Palette::fallback(theme));
        let (dark, light) = match theme {
            Theme::Dark => (active, Palette::fallback(Theme::Light)),
            Theme::Light => (Palette::fallback(Theme::Dark), active),
        };

        Self {
            theme,
            dark,
            light,
            font: desktop.font,
            fixed_font: desktop.fixed_font,
            icon_theme: desktop.icon_theme,
            reduced_motion,
        }
    }
}

const fn theme_of(dark: bool) -> Theme {
    if dark { Theme::Dark } else { Theme::Light }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is checked to be 0 to 255 first, and no From exists for f64 to u8"
)]
fn unit_to_byte(unit: f64) -> Option<u8> {
    (0.0..=1.0)
        .contains(&unit)
        .then(|| (unit * 255.0).round() as u8)
}

fn readable_on(background: Color32) -> Color32 {
    if background.intensity() > 0.5 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}
