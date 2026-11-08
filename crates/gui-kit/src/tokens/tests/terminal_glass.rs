//! The Console's terminal over the live blur: a neutral layer as thin as
//! its text and the colours programs print in keep 4.5:1, laid on the
//! sheet's frost over the app's own surfaces. Those colours are moved in
//! lightness until they read there and on the solid terminal.

use egui::{Color32, Theme};
use toolbox_terminal::Color;

use super::super::{Tokens, contrast, mix};
use super::{accents, laid_on};
use crate::oklch::{hue_chroma, lightness};
use crate::palette::Palette;
use crate::terminal_colors::{Ansi, PRINTED, resolve};

fn every_theme() -> Vec<(Theme, Palette, Tokens)> {
    let fallback = [Theme::Dark, Theme::Light].map(|theme| (theme, Palette::fallback(theme)));
    let picked = accents().map(|(theme, accent)| {
        let mut palette = Palette::fallback(theme);
        palette.accent = accent;
        (theme, palette)
    });

    fallback
        .into_iter()
        .chain(picked)
        .map(|(theme, palette)| (theme, palette, Tokens::derive(&palette, theme)))
        .collect()
}

fn view(palette: &Palette, tokens: &Tokens) -> Palette {
    Palette {
        view: tokens.terminal,
        ..*palette
    }
}

fn ansi_colour(ansi: &Ansi, palette: &Palette, tokens: &Tokens, index: u8) -> Color32 {
    resolve(Color::Indexed(index), ansi, &view(palette, tokens))
}

fn printed(palette: &Palette, tokens: &Tokens) -> Vec<(String, Color32)> {
    let colours = PRINTED.iter().map(|index| {
        (
            format!("ANSI {index}"),
            ansi_colour(&tokens.terminal_ansi, palette, tokens, *index),
        )
    });

    std::iter::once(("text".to_owned(), palette.text))
        .chain(colours)
        .collect()
}

#[test]
fn the_terminal_glass_is_neutral_black_in_dark_and_white_in_light() {
    for (theme, _, tokens) in every_theme() {
        let glass = tokens.terminal_glass;
        let shade = match theme {
            Theme::Dark => 0,
            Theme::Light => glass.a(),
        };

        assert_eq!(
            [glass.r(), glass.g(), glass.b()],
            [shade; 3],
            "{theme:?}: {glass:?} casts a tint"
        );
    }
}

#[test]
fn the_terminal_glass_is_thin_and_still_sets_the_box_off() {
    for (theme, _, tokens) in every_theme() {
        let alpha = tokens.terminal_glass.a();
        assert!(
            (51..=77).contains(&alpha),
            "{theme:?} {:?}: alpha {alpha}",
            tokens.accent
        );
    }
}

#[test]
fn printed_colours_read_on_the_solid_terminal_and_on_the_glass() {
    for (theme, palette, tokens) in every_theme() {
        let own = [
            tokens.window,
            tokens.sidebar,
            tokens.card,
            tokens.field,
            tokens.hover,
            tokens.pressed,
            mix(tokens.card, tokens.accent, 0.5),
        ];

        let glassy =
            own.map(|backdrop| laid_on(tokens.terminal_glass, laid_on(tokens.frost, backdrop)));

        for (name, ink) in printed(&palette, &tokens) {
            for backdrop in std::iter::once(tokens.terminal).chain(glassy) {
                let ratio = contrast(ink, backdrop);
                assert!(
                    ratio >= 4.5,
                    "{theme:?} {name} over {backdrop:?}: {ratio:.2}"
                );
            }
        }
    }
}

#[test]
fn printed_colours_move_in_lightness_only_and_away_from_the_terminal() {
    for (theme, palette, tokens) in every_theme() {
        let desktop = Ansi::of(&view(&palette, &tokens), theme == Theme::Dark);
        for index in PRINTED {
            let was = ansi_colour(&desktop, &palette, &tokens, index);
            let now = ansi_colour(&tokens.terminal_ansi, &palette, &tokens, index);
            let ((hue, _), (moved, _)) = (hue_chroma(was), hue_chroma(now));
            let turned = (moved - hue).rem_euclid(360.0);

            assert!(
                turned.min(360.0 - turned) < 6.0,
                "{theme:?} ANSI {index}: hue {hue} turned to {moved}"
            );

            let lighter = lightness(now) >= lightness(was) - 0.005;
            let darker = lightness(now) <= lightness(was) + 0.005;
            let away = match theme {
                Theme::Dark => lighter,
                Theme::Light => darker,
            };

            assert!(away, "{theme:?} ANSI {index}: {was:?} moved to {now:?}");
        }
    }
}

#[test]
fn a_colour_that_reads_already_is_kept() {
    let theme = Theme::Dark;
    let palette = Palette::fallback(theme);
    let tokens = Tokens::derive(&palette, theme);
    let desktop = Ansi::of(&view(&palette, &tokens), true);

    // The dark fallback's green reaches 12:1 on the terminal.
    assert_eq!(
        ansi_colour(&tokens.terminal_ansi, &palette, &tokens, 2),
        ansi_colour(&desktop, &palette, &tokens, 2)
    );
}
