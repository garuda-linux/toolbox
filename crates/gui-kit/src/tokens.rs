//! The design tokens every widget is drawn with, derived from the
//! desktop's [`Palette`].
//!
//! The desktop decides the hue: its window colour, text and accent. The
//! tokens decide the steps between surfaces and the contrast of text, so
//! the app keeps its depth and stays readable on any desktop theme.

mod frost;
pub mod letters;
mod shades;
pub mod sizes;
mod terminal;
#[cfg(test)]
mod tests;
pub mod tiles;
mod wash;

use egui::{Color32, Context, Id, Rgba, Theme};
pub use letters::{LETTER_HUES, LetterTiles};
pub use sizes::Sizes;
pub use tiles::{Hue, Tile, Tiles, page_hue, set_page_hue};

use self::frost::{thin, veil};
use self::shades::{shades, track_hover};
use crate::palette::Palette;
use crate::terminal_colors::Ansi;

const TEXT_CONTRAST: f32 = 7.0;
/// WCAG AA for body text.
const SECONDARY_CONTRAST: f32 = 4.5;
const LIGHT_SECONDARY_CONTRAST: f32 = 6.0;
/// WCAG 1.4.11 for text no one reads and the edge of a control.
pub(crate) const DIM_CONTRAST: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub window: Color32,
    /// The header's scrim on the window, so sidebar and header read as one surface.
    pub sidebar: Color32,
    pub sidebar_text: Color32,
    /// The sidebar's colour outside its column, on the status bar and the Console sheet.
    pub pane: Color32,
    pub card: Color32,
    pub raised: Color32,
    pub field: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub outline: Color32,
    pub sidebar_edge: Color32,
    /// At least 3:1 against the field fill, so a button's shape reads on a banner.
    pub rim: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_pressed: Color32,
    pub tint_pressed: Color32,
    pub accent_soft: Color32,
    pub accent_tint: Color32,
    /// The accent moved until it reads on the tint and on its pressed shade.
    pub on_tint: Color32,
    pub on_accent: Color32,
    /// The accent moved until it reads at 3:1 on every fill it sits on.
    pub accent_ring: Color32,
    /// Links and figures in the accent, moved to 4.5:1.
    pub accent_text: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub on_error: Color32,
    pub error_hover: Color32,
    pub error_pressed: Color32,
    pub track_hover: Color32,
    pub terminal: Color32,
    /// Thin neutral black or white, so the blur casts no tint and output stays readable.
    pub terminal_glass: Color32,
    /// Program colours, moved in lightness until they read on the terminal and its glass.
    pub terminal_ansi: Ansi,
    /// The pane colour, see-through only as far as text on it stays readable.
    pub frost: Color32,
    /// Thin neutral black or white, so the blur shows and casts no tint.
    pub header_scrim: Color32,
    /// No blur lies behind them, so what is under shows only as a faint ghost.
    pub frost_raised: Color32,
    /// Thinner than `frost_raised`, as the blur under it shows through.
    pub frost_raised_blurred: Color32,
    pub shadow: Color32,
    pub tiles: Tiles,
    pub sizes: Sizes,
}

impl Tokens {
    pub fn derive(palette: &Palette, theme: Theme) -> Self {
        let (base, ink) = (palette.window, palette.text);
        let surfaces = Surfaces::of(base, ink, theme);

        let (card, accent) = (surfaces.card, palette.accent);
        let text = readable(ink, card, TEXT_CONTRAST);

        let error = fitted(palette.error, &surfaces.under_text(), SECONDARY_CONTRAST);
        let secondary = surfaces.secondary(text, theme);

        let (tint, tint_pressed) = tints(card, accent, theme);
        let header_scrim = surfaces.scrim(theme, [text, secondary]);

        // Text that reads on the window under the scrim reads on this too,
        // as the scrim only moves the window away from the text.
        let sidebar = surfaces.window.blend(header_scrim);

        let soft = mix(card, accent, 0.16);
        let on_error = readable(card, error, SECONDARY_CONTRAST);

        let (accent_hover, accent_pressed, terminal, shadow) =
            shades(theme, accent, &surfaces, base, ink);

        let (terminal_glass, terminal_ansi) =
            surfaces.terminal_look(theme, palette, [text, terminal], sidebar);

        Self {
            window: surfaces.window,
            sidebar,
            sidebar_text: secondary,
            pane: surfaces.pane,
            card,
            raised: card,
            field: surfaces.field,
            hover: surfaces.hover,
            pressed: surfaces.pressed,
            outline: surfaces.outline,
            sidebar_edge: surfaces.outline,
            rim: readable(mix(text, card, 0.5), surfaces.field, DIM_CONTRAST),
            text,
            secondary,
            dim: readable(mix(text, card, 0.5), card, DIM_CONTRAST),
            accent,
            accent_hover,
            accent_pressed,
            accent_soft: soft,
            accent_tint: tint,
            tint_pressed,
            on_tint: fitted(accent, &[tint, tint_pressed], SECONDARY_CONTRAST),
            on_accent: readable(palette.on_accent, accent, SECONDARY_CONTRAST),
            accent_ring: fitted(accent, &surfaces.under(sidebar, tint), DIM_CONTRAST),
            accent_text: fitted(accent, &[surfaces.window, card, soft], SECONDARY_CONTRAST),
            success: fitted(palette.success, &surfaces.under_text(), SECONDARY_CONTRAST),
            warning: fitted(palette.warning, &surfaces.under_text(), SECONDARY_CONTRAST),
            error,
            on_error,
            error_hover: mix(error, on_error, 0.12),
            error_pressed: mix(error, on_error, 0.22),
            track_hover: track_hover(theme, surfaces.pressed, text),
            terminal,
            terminal_glass,
            terminal_ansi,
            frost: surfaces.frost(text, sidebar, accent),
            header_scrim,
            frost_raised: veil(card, secondary),
            frost_raised_blurred: thin(card, secondary, &surfaces.under_blur(sidebar, accent)),
            shadow,
            tiles: Tiles::derive(card, palette, text, theme),
            sizes: Sizes::DEFAULT,
        }
    }
}

struct Surfaces {
    window: Color32,
    /// The status bar and the Console sheet: a step lighter than the
    /// window when dark, the desktop's own colour when light.
    pane: Color32,
    card: Color32,
    field: Color32,
    hover: Color32,
    pressed: Color32,
    outline: Color32,
}

impl Surfaces {
    /// Descriptions and hints: as [`description`] finds them, then moved
    /// until they read on every fill status words sit on.
    fn secondary(&self, text: Color32, theme: Theme) -> Color32 {
        let first = description(text, self.card, self.window, secondary_contrast(theme));
        fitted(first, &self.under_text(), SECONDARY_CONTRAST)
    }

    /// The sidebar and the status bar follow from the window.
    const fn under_text(&self) -> [Color32; 4] {
        [self.card, self.window, self.field, self.hover]
    }

    const fn under(&self, sidebar: Color32, tint: Color32) -> [Color32; 5] {
        [self.window, sidebar, self.card, self.field, tint]
    }

    /// A blur this wide never leaves a pill-sized accent at full strength.
    fn under_blur(&self, sidebar: Color32, accent: Color32) -> [Color32; 7] {
        [
            self.window,
            sidebar,
            self.card,
            self.field,
            self.hover,
            self.pressed,
            mix(self.card, accent, 0.5),
        ]
    }

    fn of(base: Color32, ink: Color32, theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                window: mix(base, Color32::BLACK, 0.35),
                pane: mix(base, Color32::BLACK, 0.2),
                card: mix(base, Color32::BLACK, 0.08),
                field: mix(base, ink, 0.03),
                hover: mix(base, ink, 0.08),
                pressed: mix(base, ink, 0.14),
                outline: mix(base, ink, 0.1),
            },
            Theme::Light => Self {
                window: mix(base, Color32::WHITE, 0.55),
                pane: base,
                card: Color32::WHITE,
                field: mix(base, ink, 0.05),
                hover: mix(base, ink, 0.1),
                pressed: mix(base, ink, 0.16),
                outline: mix(base, ink, 0.12),
            },
        }
    }
}

pub fn store(ctx: &Context, dark: Tokens, light: Tokens) {
    ctx.data_mut(|data| data.insert_temp(tokens_id(), (dark, light)));
}

/// The tokens of the theme in use. Before any were stored, the fallback
/// palette's.
pub fn current(ctx: &Context) -> Tokens {
    let theme = ctx.theme();
    ctx.data(|data| data.get_temp::<(Tokens, Tokens)>(tokens_id()))
        .map_or_else(
            || Tokens::derive(&Palette::fallback(theme), theme),
            |(dark, light)| match theme {
                Theme::Dark => dark,
                Theme::Light => light,
            },
        )
}

/// `from` moved toward `to` by `amount` (0 to 1), in gamma space as
/// designers mix.
pub fn mix(from: Color32, to: Color32, amount: f32) -> Color32 {
    from.lerp_to_gamma(to, amount)
}

pub fn contrast(first: Color32, second: Color32) -> f32 {
    let (lighter, darker) = {
        let (one, other) = (luminance(first), luminance(second));
        if one > other {
            (one, other)
        } else {
            (other, one)
        }
    };

    (lighter + 0.05) / (darker + 0.05)
}

/// The text of descriptions and hints. Page headers sit on the window,
/// which light themes draw darker than the cards, so it reads on both.
fn description(text: Color32, card: Color32, window: Color32, ratio: f32) -> Color32 {
    readable(readable(mix(text, card, 0.3), card, ratio), window, ratio)
}

/// What secondary text reaches on cards and the window. Light themes keep
/// headroom above [`SECONDARY_CONTRAST`], or a frosted surface over the
/// blur would have to be near opaque to keep it readable.
const fn secondary_contrast(theme: Theme) -> f32 {
    match theme {
        Theme::Dark => SECONDARY_CONTRAST,
        Theme::Light => LIGHT_SECONDARY_CONTRAST,
    }
}

fn tints(card: Color32, accent: Color32, theme: Theme) -> (Color32, Color32) {
    let tint = mix(card, accent, tint_share(theme));
    (tint, mix(tint, accent, 0.15))
}

/// How much accent the selection tint takes. Light themes need more to
/// stand out as much as the dark one.
const fn tint_share(theme: Theme) -> f32 {
    match theme {
        Theme::Dark => 0.10,
        Theme::Light => 0.24,
    }
}

/// `color` moved until it reaches `ratio` on each of `fills`, which share
/// a side of the scale, so each move keeps the earlier ones.
fn fitted(color: Color32, fills: &[Color32], ratio: f32) -> Color32 {
    fills
        .iter()
        .fold(color, |moved, fill| readable(moved, *fill, ratio))
}

fn readable(color: Color32, background: Color32, ratio: f32) -> Color32 {
    let target = if luminance(background) > 0.18 {
        Color32::BLACK
    } else {
        Color32::WHITE
    };

    (0_u8..=20)
        .map(|step| mix(color, target, f32::from(step) / 20.0))
        .find(|candidate| contrast(*candidate, background) >= ratio)
        .unwrap_or(target)
}

fn luminance(color: Color32) -> f32 {
    let linear = Rgba::from(color);
    0.0722_f32.mul_add(
        linear.b(),
        0.2126_f32.mul_add(linear.r(), 0.7152 * linear.g()),
    )
}

fn tokens_id() -> Id {
    Id::unique("toolbox_gui_kit::tokens")
}
