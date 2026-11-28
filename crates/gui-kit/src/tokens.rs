//! The design tokens every widget is drawn with, derived from the
//! desktop's [`Palette`].
//!
//! The desktop decides the hue: its window colour, text and accent. The
//! tokens decide the steps between surfaces and the contrast of text, so
//! the app keeps its depth and stays readable on any desktop theme.

mod card;
mod code;
pub(crate) mod darkly;
mod frost;
mod glass;
pub mod letters;
mod model;
mod readability;
mod roles;
mod shade;
mod shades;
pub mod sizes;
mod terminal;
#[cfg(test)]
mod tests;
pub mod tiles;
mod wash;

use egui::{Color32, Context, Id, Theme};
pub use letters::{LETTER_HUES, LetterTiles};
pub use model::Tokens;
pub use readability::contrast;
pub use sizes::Sizes;
pub use tiles::{Hue, Tile, Tiles, page_hue, set_page_hue};

use self::card::form_border;
pub use self::card::is_dark_surface;
pub use self::code::CodeInks;
use self::frost::{thin, veil};
pub use self::glass::WINDOW_OPACITY;
use self::readability::{Ends, description, fitted, readable, toward_ends};
pub use self::roles::Roles;
use self::shades::{shades, shimmer, top_light};
use crate::palette::Palette;

const TEXT_CONTRAST: f32 = 7.0;
/// The line between the rows of a card, as Kirigami's separator at half
/// opacity: the share of the text mixed into the card.
pub(crate) const ROW_DIVIDER_SHARE: f32 = 0.085;
/// WCAG AA for body text.
const SECONDARY_CONTRAST: f32 = 4.5;
const LIGHT_SECONDARY_CONTRAST: f32 = 6.0;
/// WCAG 1.4.11 for text no one reads and the edge of a control.
pub(crate) const DIM_CONTRAST: f32 = 3.0;
/// The selection text the desktop names keeps its colour down to this
/// contrast on the accent, as a selected row's label is body text. Below
/// it the theme's text or window takes over, fitted to it.
const SELECTION_LEAST: f32 = SECONDARY_CONTRAST;
/// The dialog scrim's darkness. The blurred light window turns muddy grey
/// under more, the dark one needs more to step back behind the dialog.
const LIGHT_DIALOG_SCRIM: f32 = 0.25;
const DARK_DIALOG_SCRIM: f32 = 0.45;

impl Tokens {
    #[expect(
        clippy::too_many_lines,
        reason = "one line per token, which groups would only hide"
    )]
    pub fn derive(palette: &Palette, theme: Theme) -> Self {
        let (base, ink) = (palette.window, palette.text);
        let surfaces = Surfaces::of(palette);

        let (card, accent) = (surfaces.card, palette.accent);
        let shadow_shade = shade::shadow_shade(base, palette.contrast);

        // The text is fitted first toward the raw desktop text, so the ends
        // the other colours move between are the theme's own text and window.
        let raw_ends = Ends {
            text: ink,
            window: base,
            shadow: shadow_shade,
        };

        let text = readable(ink, card, TEXT_CONTRAST, raw_ends);

        let ends = Ends {
            text,
            window: base,
            shadow: shadow_shade,
        };

        let secondary = surfaces.secondary(text, theme, ends);

        let header_scrim = surfaces.scrim(theme, [text, secondary], ends);

        // Text that reads on the window under the scrim reads on this too,
        // as the scrim only moves the window away from the text.
        let sidebar = surfaces.window.blend(header_scrim);

        let soft = mix(card, accent, 0.16);
        let error = error_colour(palette.error, &surfaces, ends);

        let (tint_pressed, terminal, shadow) = shades(theme, accent, &surfaces, ends);

        let (terminal_glass, terminal_ansi) =
            surfaces.terminal_look(theme, palette, [text, terminal], sidebar, ends);

        let card_outline = mix(surfaces.window, text, 0.25);
        let control_shadow = darkly::alpha(shadow_shade, 40);
        let input = card::input_fill(surfaces.window, palette.view, ink);
        let button_hover = darkly::lighter(input, 1.05);
        let decoration_focus = palette.decoration_focus.unwrap_or(accent);
        let highlighted_text = selection_text(palette.on_accent, accent, ends);
        let focus_fill = text_focus_fill(decoration_focus, text, ends);
        let on_focus = fitted(
            text,
            &[focus_fill, focus_hovered(focus_fill)],
            SECONDARY_CONTRAST,
            ends,
        );

        let tooltip = palette.tooltip.unwrap_or(card);

        Self {
            window: surfaces.window,
            window_glass: surfaces.window,
            band: surfaces.window,
            chosen_tab: input,
            selection: accent,
            selection_text: highlighted_text,
            sidebar,
            sidebar_fade: sidebar,
            sidebar_text: text,
            pane: surfaces.pane,
            card,
            card_glass: card,
            card_edge: card_outline,
            card_shadow: control_shadow,
            row_divider: mix(card, text, ROW_DIVIDER_SHARE),
            raised: card,
            field: surfaces.field,
            view: palette.view,
            input,
            field_on_raised: surfaces.field_on_raised,
            button_hover,
            button_pressed: darkly::darker(decoration_focus, 1.1),
            default_button: mix(input, palette.view, 0.7),
            // Darkly's list hover is the highlight at 0.2 (darklystyle.cpp:4768).
            item_hover: accent.gamma_multiply(0.2),
            popup_hover: accent.gamma_multiply(0.2),
            tint_hover: darkly::lighter(accent, 1.1),
            outline: surfaces.outline,
            rule: surfaces.outline,
            card_outline,
            separator: darkly::separator(base, text, shadow_shade),
            sidebar_edge: darkly::sidebar_edge(base, shadow_shade),
            shadow_shade,
            rim: readable(mix(text, card, 0.5), surfaces.field, DIM_CONTRAST, ends),
            text,
            secondary,
            dim: readable(mix(text, card, 0.5), card, DIM_CONTRAST, ends),
            accent,
            accent_soft: soft,
            accent_tint: accent,
            tint_pressed,
            highlighted_text,
            on_tint: highlighted_text,
            on_accent: readable(palette.on_accent, accent, SECONDARY_CONTRAST, ends),
            decoration_hover: palette.decoration_hover.unwrap_or(accent),
            decoration_focus,
            focus_fill,
            on_focus,
            tooltip,
            tooltip_text: readable(
                palette.tooltip_text.unwrap_or(text),
                tooltip,
                TEXT_CONTRAST,
                ends,
            ),
            accent_ring: accent,
            accent_text: fitted(
                accent,
                &[surfaces.window, card, soft],
                SECONDARY_CONTRAST,
                ends,
            ),
            success: fitted(
                palette.success,
                &surfaces.under_text(),
                SECONDARY_CONTRAST,
                ends,
            ),
            warning: fitted(
                palette.warning,
                &surfaces.under_text(),
                SECONDARY_CONTRAST,
                ends,
            ),
            roles: roles::derive(palette, &surfaces, accent, ink, ends),
            error,
            top_light: top_light(theme, card, ends.light()),
            button_top_light: darkly::top_highlight(base, ends.light()),
            shimmer: shimmer(theme, button_hover, ends),
            terminal,
            terminal_glass,
            terminal_ansi,
            code: CodeInks::derive(&terminal_ansi, palette, [text, surfaces.field], ends),
            frost: surfaces.frost(text, sidebar, accent),
            header_scrim,
            frost_raised: veil(card, secondary, [ends.light(), ends.shadow]),
            frost_raised_blurred: thin(card, secondary, &surfaces.under_blur(sidebar, accent)),
            scrim: dialog_scrim(theme, shadow_shade),
            shadow,
            control_shadow,
            tiles: Tiles::derive(card, palette, ends, theme),
            sizes: Sizes::DEFAULT,
        }
    }
}

struct Surfaces {
    window: Color32,
    /// The status bar and the Console sheet: the window colour in both themes.
    pane: Color32,
    card: Color32,
    field: Color32,
    field_on_raised: Color32,
    hover: Color32,
    pressed: Color32,
    outline: Color32,
}

impl Surfaces {
    /// Descriptions and hints: as [`description`] finds them, then moved
    /// until they read on every fill status words sit on.
    fn secondary(&self, text: Color32, theme: Theme, ends: Ends) -> Color32 {
        let first = description(
            text,
            self.card,
            self.window,
            secondary_contrast(theme),
            ends,
        );

        fitted(first, &self.under_text(), SECONDARY_CONTRAST, ends)
    }

    /// The fills that status words and descriptions sit on.
    const fn under_text(&self) -> [Color32; 4] {
        [self.card, self.window, self.field, self.hover]
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

    fn of(palette: &Palette) -> Self {
        let (base, view, ink) = (palette.window, palette.view, palette.text);
        let field = palette.button.unwrap_or_else(|| mix(view, ink, 0.03));
        Self::from_desktop(base, view, field, ink)
    }

    /// The desktop's own window, view and button colours, as Qt paints
    /// them. Hover and press step up from the button. A card is Darkly's
    /// group box fill, see [`darkly::frame_fill`].
    fn from_desktop(base: Color32, view: Color32, field: Color32, ink: Color32) -> Self {
        Self {
            window: base,
            pane: base,
            card: card::card_fill(base, view, ink),
            field,
            field_on_raised: field,
            hover: mix(field, ink, 0.06),
            pressed: mix(field, ink, 0.12),
            outline: form_border(view, ink),
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
pub(crate) fn mix(from: Color32, to: Color32, amount: f32) -> Color32 {
    from.lerp_to_gamma(to, amount)
}

/// The fill a focused control with text takes under the pointer: lighter
/// than its rest fill, as Darkly lightens a focused button.
fn focus_hovered(fill: Color32) -> Color32 {
    darkly::lighter(darkly::lighter(fill, 1.15), 1.02)
}

/// The keyboard focus fill of a control that carries text. The desktop's
/// focus colour where text moved toward the theme's ends reaches 4.5:1 on
/// it at rest and under the pointer, else the colour darkened toward the
/// shadow shade in steps until it does, so light text can sit on it. A text
/// relit past the ends would take any fill and leave a light text dark.
fn text_focus_fill(decoration_focus: Color32, text: Color32, ends: Ends) -> Color32 {
    let reads = |fill: Color32| {
        [fill, focus_hovered(fill)]
            .into_iter()
            .all(|on| toward_ends(text, on, SECONDARY_CONTRAST, ends).is_some())
    };

    (0..=20_u8)
        .map(|step| mix(decoration_focus, ends.shadow, f32::from(step) / 20.0))
        .find(|fill| reads(*fill))
        .unwrap_or_else(|| mix(decoration_focus, ends.shadow, 1.0))
}

/// The desktop's selection text where it reads on the accent, else whichever
/// of the theme's text and window reads better on it, fitted to 4.5:1 where
/// neither reaches it.
fn selection_text(on_accent: Color32, accent: Color32, ends: Ends) -> Color32 {
    if contrast(on_accent, accent) >= SELECTION_LEAST {
        return on_accent;
    }

    let better = if contrast(ends.text, accent) >= contrast(ends.window, accent) {
        ends.text
    } else {
        ends.window
    };

    readable(better, accent, SECONDARY_CONTRAST, ends)
}

/// The shadow shade at each theme's share, so the window steps back behind the dialog.
fn dialog_scrim(theme: Theme, shadow: Color32) -> Color32 {
    let share = match theme {
        Theme::Dark => DARK_DIALOG_SCRIM,
        Theme::Light => LIGHT_DIALOG_SCRIM,
    };

    shadow.gamma_multiply(share)
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

/// The error colour fitted to every surface text sits on.
fn error_colour(error: Color32, surfaces: &Surfaces, ends: Ends) -> Color32 {
    fitted(error, &surfaces.under_text(), SECONDARY_CONTRAST, ends)
}

fn tokens_id() -> Id {
    Id::unique("toolbox_gui_kit::tokens")
}
