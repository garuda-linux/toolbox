#![expect(
    clippy::disallowed_methods,
    reason = "tests build desktop palettes from literal colours"
)]

use egui::{Color32, Theme};

use super::super::letters::LETTER_HUES;
use super::super::{Tokens, contrast};
use crate::oklch::hue_chroma;
use crate::palette::{Palette, letter_hues};

fn breeze_dark() -> Palette {
    Palette {
        window: Color32::from_rgb(32, 35, 38),
        view: Color32::from_rgb(20, 22, 24),
        text: Color32::from_rgb(252, 252, 252),
        accent: Color32::from_rgb(61, 174, 233),
        on_accent: Color32::from_rgb(252, 252, 252),
        error: Color32::from_rgb(218, 68, 83),
        warning: Color32::from_rgb(246, 116, 0),
        success: Color32::from_rgb(39, 174, 96),
        link: Some(Color32::from_rgb(29, 153, 243)),
        visited: Some(Color32::from_rgb(155, 89, 182)),
        tiles: None,
        ansi: None,
    }
}

fn breeze_light() -> Palette {
    Palette {
        window: Color32::from_rgb(239, 240, 241),
        view: Color32::from_rgb(255, 255, 255),
        text: Color32::from_rgb(35, 38, 41),
        link: Some(Color32::from_rgb(41, 128, 185)),
        ..breeze_dark()
    }
}

/// A scheme whose every colour is one blue, so the tiles fill up with
/// turns of the accent.
fn one_hue() -> Palette {
    let blue = Color32::from_rgb(0x1e, 0x66, 0xf5);
    Palette {
        accent: blue,
        error: blue,
        warning: blue,
        success: blue,
        link: Some(blue),
        visited: Some(blue),
        ..Palette::fallback(Theme::Light)
    }
}

/// A grey accent and grey status colours, whose hue says nothing.
pub(super) fn grey() -> Palette {
    let grey = Color32::from_rgb(0x80, 0x80, 0x80);
    Palette {
        accent: grey,
        error: grey,
        warning: grey,
        success: grey,
        ..Palette::fallback(Theme::Dark)
    }
}

fn cases() -> [(&'static str, Theme, Tokens); 7] {
    [
        ("fallback", Palette::fallback(Theme::Dark), Theme::Dark),
        ("fallback", Palette::fallback(Theme::Light), Theme::Light),
        ("breeze dark", breeze_dark(), Theme::Dark),
        ("breeze light", breeze_light(), Theme::Light),
        ("one hue", one_hue(), Theme::Light),
        ("one hue", one_hue(), Theme::Dark),
        ("grey", grey(), Theme::Dark),
    ]
    .map(|(name, palette, theme)| (name, theme, Tokens::derive(&palette, theme)))
}

pub(super) fn fills(tokens: &Tokens) -> Vec<Color32> {
    (0..LETTER_HUES)
        .map(|index| tokens.tiles.letters().of(index).fill)
        .collect()
}

fn hue_angles(tokens: &Tokens) -> Vec<f32> {
    fills(tokens)
        .into_iter()
        .map(|fill| hue_chroma(fill).0)
        .collect()
}

/// The angle between two hues, 0 to 180.
fn gap(one: f32, other: f32) -> f32 {
    let turn = (one - other).rem_euclid(360.0);
    turn.min(360.0 - turn)
}

#[test]
fn every_letter_reads_at_four_and_a_half_to_one_on_its_tile() {
    for (name, theme, tokens) in cases() {
        for index in 0..LETTER_HUES {
            let tile = tokens.tiles.letters().of(index);
            let ratio = contrast(tile.ink, tile.fill);
            assert!(ratio >= 4.5, "{name} {theme:?} hue {index}: {ratio:.2}:1");
        }
    }
}

#[test]
fn the_letter_is_white_or_the_text_whichever_reads_better() {
    for (name, theme, tokens) in cases() {
        for index in 0..LETTER_HUES {
            let tile = tokens.tiles.letters().of(index);
            let best = [Color32::WHITE, tokens.text]
                .into_iter()
                .map(|ink| contrast(ink, tile.fill))
                .fold(0.0_f32, f32::max);

            assert!(
                (contrast(tile.ink, tile.fill) - best).abs() < f32::EPSILON,
                "{name} {theme:?} hue {index}: the ink is not the better of the two"
            );
        }
    }
}

#[test]
fn the_fills_are_eight_hues_set_apart() {
    assert_eq!(LETTER_HUES, 8);

    for (name, theme, tokens) in cases() {
        let hues = hue_angles(&tokens);

        for (index, hue) in hues.iter().enumerate() {
            for other in hues.iter().skip(index + 1) {
                assert!(
                    gap(*hue, *other) >= 12.0,
                    "{name} {theme:?}: hues {hue:.0} and {other:.0} are too close"
                );
            }
        }
    }
}

#[test]
fn a_letter_tile_stands_out_from_the_card_of_its_row() {
    for (name, theme, tokens) in cases() {
        for (index, fill) in fills(&tokens).into_iter().enumerate() {
            let ratio = contrast(fill, tokens.card);
            assert!(ratio >= 1.5, "{name} {theme:?} hue {index}: {ratio:.2}:1");
        }
    }
}

#[test]
fn without_a_desktop_scheme_the_fallback_hues_are_kept() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        assert_eq!(fills(&tokens), letter_hues(theme));
    }
}

#[test]
fn the_scheme_s_own_colours_lead_the_hues() {
    let palette = breeze_dark();
    let tokens = Tokens::derive(&palette, Theme::Dark);
    let hues = hue_angles(&tokens);

    let named = [
        ("accent", palette.accent),
        ("positive", palette.success),
        ("neutral", palette.warning),
        ("negative", palette.error),
    ];

    for (index, (role, colour)) in named.into_iter().enumerate() {
        let (wanted, _) = hue_chroma(colour);
        let hue = hues.get(index).copied().unwrap_or_default();
        assert!(
            gap(hue, wanted) <= 6.0,
            "tile {index} is at {hue:.0}, the {role} colour at {wanted:.0}"
        );
    }
}

#[test]
fn another_scheme_gives_other_fills() {
    let dark = Tokens::derive(&breeze_dark(), Theme::Dark);
    let fallback = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark);

    assert_ne!(fills(&dark), fills(&fallback));
}

#[test]
fn an_index_past_the_palette_wraps_round() {
    let (_, _, tokens) = cases()[0];
    assert_eq!(
        tokens.tiles.letters().of(LETTER_HUES + 2),
        tokens.tiles.letters().of(2)
    );
}
