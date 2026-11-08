//! The colours by role, which the app derives its [`crate::tokens`] from.
//!
//! The desktop readers fill them from `kdeglobals` or GTK,
//! [`Palette::fallback`] stands in when neither can be read, and one of
//! the app's own schemes picked in its settings takes the desktop's place.

#[cfg(test)]
mod tests;

use egui::{Color32, Context, Id, Theme};

use crate::terminal_colors::Ansi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub window: Color32,
    /// Text fields, lists and the console, set off from the window.
    pub view: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,
    /// The scheme's link and visited link colours, which only some
    /// desktops name. They give the letter tiles more of the scheme's hues.
    pub link: Option<Color32>,
    pub visited: Option<Color32>,
    /// The icon tiles' hues of one of the app's own schemes. A desktop
    /// names none, so `None` takes Catppuccin's ([`TileHues::of`]).
    pub tiles: Option<TileHues>,
    /// The terminal's 16 colours of one of the app's own schemes. `None`
    /// derives them from the roles ([`Ansi::of`]).
    pub ansi: Option<Ansi>,
}

impl Palette {
    /// Catppuccin Mocha for dark and Latte for light, both with the mauve
    /// accent.
    #[expect(
        clippy::disallowed_methods,
        reason = "the palette and the desktop parsers are where colours are made"
    )]
    #[must_use = "a palette does nothing until it is applied"]
    pub const fn fallback(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                window: Color32::from_rgb(0x1e, 0x1e, 0x2e),
                view: Color32::from_rgb(0x18, 0x18, 0x25),
                text: Color32::from_rgb(0xcd, 0xd6, 0xf4),
                accent: Color32::from_rgb(0xcb, 0xa6, 0xf7),
                on_accent: Color32::from_rgb(0x11, 0x11, 0x1b),
                error: Color32::from_rgb(0xf3, 0x8b, 0xa8),
                warning: Color32::from_rgb(0xf9, 0xe2, 0xaf),
                success: Color32::from_rgb(0xa6, 0xe3, 0xa1),
                link: None,
                visited: None,
                tiles: None,
                ansi: None,
            },
            Theme::Light => Self {
                window: Color32::from_rgb(0xef, 0xf1, 0xf5),
                view: Color32::from_rgb(0xe6, 0xe9, 0xef),
                text: Color32::from_rgb(0x4c, 0x4f, 0x69),
                accent: Color32::from_rgb(0x88, 0x39, 0xef),
                on_accent: Color32::from_rgb(0xef, 0xf1, 0xf5),
                error: Color32::from_rgb(0xd2, 0x0f, 0x39),
                warning: Color32::from_rgb(0xdf, 0x8e, 0x1d),
                success: Color32::from_rgb(0x40, 0xa0, 0x2b),
                link: None,
                visited: None,
                tiles: None,
                ansi: None,
            },
        }
    }
}

/// The hues of the tiles behind row icons, one per part of the app. The
/// desktop names no such colours, so Catppuccin's stand for every theme
/// without a scheme of the app's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileHues {
    pub blue: Color32,
    pub green: Color32,
    pub orange: Color32,
}

impl TileHues {
    /// Mocha's blue, green and peach for dark, Latte's for light.
    #[expect(
        clippy::disallowed_methods,
        reason = "the palette is where colours are made"
    )]
    pub const fn of(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                blue: Color32::from_rgb(0x89, 0xb4, 0xfa),
                green: Color32::from_rgb(0xa6, 0xe3, 0xa1),
                orange: Color32::from_rgb(0xfa, 0xb3, 0x87),
            },
            Theme::Light => Self {
                blue: Color32::from_rgb(0x1e, 0x66, 0xf5),
                green: Color32::from_rgb(0x40, 0xa0, 0x2b),
                orange: Color32::from_rgb(0xfe, 0x64, 0x0b),
            },
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the palette is where colours are made, from the scheme catalog too"
)]
pub const fn colour([red, green, blue]: [u8; 3]) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

/// The letter tiles' fills while no desktop scheme was read, and the
/// lightness and chroma a scheme's own hues take.
///
/// Red, orange, olive, green, teal, blue, indigo and purple at one OKLCH
/// lightness, muted so a page of them stays calm, each dark enough for a
/// white letter at 4.5:1. The light theme's are a step lighter.
#[expect(
    clippy::disallowed_methods,
    reason = "the palette is where colours are made"
)]
pub const fn letter_hues(theme: Theme) -> [Color32; 8] {
    match theme {
        Theme::Dark => [
            Color32::from_rgb(0x8d, 0x4f, 0x4a),
            Color32::from_rgb(0x86, 0x57, 0x2d),
            Color32::from_rgb(0x72, 0x62, 0x22),
            Color32::from_rgb(0x42, 0x70, 0x44),
            Color32::from_rgb(0x01, 0x72, 0x72),
            Color32::from_rgb(0x35, 0x68, 0x90),
            Color32::from_rgb(0x5a, 0x5d, 0x93),
            Color32::from_rgb(0x7e, 0x51, 0x7a),
        ],
        Theme::Light => [
            Color32::from_rgb(0xa0, 0x51, 0x4c),
            Color32::from_rgb(0x97, 0x5b, 0x23),
            Color32::from_rgb(0x7f, 0x6a, 0x0c),
            Color32::from_rgb(0x41, 0x7b, 0x44),
            Color32::from_rgb(0x07, 0x7b, 0x7c),
            Color32::from_rgb(0x2e, 0x71, 0xa4),
            Color32::from_rgb(0x60, 0x64, 0xa7),
            Color32::from_rgb(0x8d, 0x54, 0x89),
        ],
    }
}

/// Keeps both themes' palettes for [`current`], as
/// [`crate::theme::apply`] does.
pub fn store(ctx: &Context, dark: Palette, light: Palette) {
    ctx.data_mut(|data| data.insert_temp(palettes_id(), (dark, light)));
}

/// The desktop palette of the theme in use, for colours that follow the
/// desktop exactly, such as the terminal's.
pub fn current(ctx: &Context) -> Palette {
    let (dark, light) = both(ctx);
    match ctx.theme() {
        Theme::Dark => dark,
        Theme::Light => light,
    }
}

/// Both themes' palettes as stored, the fallback ones before that.
pub fn both(ctx: &Context) -> (Palette, Palette) {
    ctx.data(|data| data.get_temp::<(Palette, Palette)>(palettes_id()))
        .unwrap_or_else(|| {
            (
                Palette::fallback(Theme::Dark),
                Palette::fallback(Theme::Light),
            )
        })
}

fn palettes_id() -> Id {
    Id::unique("toolbox_gui_kit::palette")
}
